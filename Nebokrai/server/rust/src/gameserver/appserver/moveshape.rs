//! Реализованная часть `CMoveShape` исторического GameServer.
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
//!
//! Известные состояния смены тела, расширенные состояния, бессмертие,
//! сценарные состояния и езда принадлежат одному `CanonicalStateStorage`.
//! Сырой `ex_states` скрыт внутри `LegacyStateCodec` и служит только для
//! сохранения точного порядка, неизвестных записей и обратного двоичного кодека;
//! игровое поведение читает типизированные состояния. Пересчёт свойств игрока
//! и монстра проецирует достигнутые property-state в том же insertion-order, включая
//! повторные экземпляры одного ID. Добавление, замена,
//! таймеры и удаление обновляют типизированную модель и её кодек в одной
//! операции с прежними смещениями и порядком.
//! Замена Cure сохраняет позицию удалённой записи через отдельный offset:
//! между End и установкой нового экземпляра слот отсутствует в проекции.
//! Vec::splice возвращает DB-запись на место, сдвигая сохранённые смещения
//! соседей без их повторной загрузки и без сброса runtime-таймеров.
//! Сбор душ хранится здесь без таймера и без параллельной raw-записи. Печать,
//! паутина и оглушение дополнительно сохраняют общий порядок вставки для
//! завершения через унаследованное защитное действие `CBlindState`.
//! `CStrikeState` хранится типизированно в общей 8-байтной DB-записи,
//! участвует в запретах движения и боя и удаляется при строгом истечении.
//! Рыцарский удар хранит здесь единственную каноническую блокировку движения
//! и боя; замена, истечение и снятие очищением меняют те же счётчики.
//! Подготовка яростного удара также имеет здесь единственный типизированный
//! экземпляр: замена, истечение и потребление `Flash` не касаются
//! скрытой устаревшей двоичной записи.
//! `PillarState` хранится здесь же: проверки рывков, строгий таймер и поздний
//! коэффициент защиты читают один экземпляр без параллельной сырой записи.
//! Оглушения `RushState` и `Rush2State` также имеют здесь независимые
//! канонические сроки и через общие счётчики управляют запретами движения и
//! боя для игрока либо регионального монстра.
//! `CNotDisappearAfterDead` использует точный client-time override
//! `CExStateNew::GetRemainedTime`: нулевой срок и достигнутый wrapping deadline
//! дают `0`, иначе публикуется оставшийся DWORD.
//! Расходуемые и автоматические восстановления HP/MP также входят в общий
//! DB-кодек. 16-байтные расходуемые записи материализуются при загрузке,
//! активируются при входе и удаляются из wire вместе с живым состоянием, не
//! обрывая следующий record. Четыре 12-байтных автоматических записи при
//! `RestoreHpMp` атомарно заменяются значениями актуальных свойств игрока.
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
//! Четыре стандартных Vec сохраняют независимые экземпляры, их порядок и
//! повторные ID: native AddSkill (0x004D1C70) допускает
//! повторный ID, когда уровень первого найденного экземпляра равен нулю.
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
//! Изменяемый доступ не создаёт фиктивного monster-kernel и не подменяет
//! чужой вариант. Общий IsEnded не выводится из наличия concrete-исполнения;
//! отдельный immediate-флаг +0x4c не подменяет базовый lifecycle.
//! CSkill constructor (0x004D8120) задаёт timestamp +0x40 равным нулю;
//! новая регистрация не наследует его от удалённого экземпляра того же ID.
//! Доступ к этим полям использует тот же первый GetSkill по текущей metadata,
//! без дополнительного реестра и без поиска по intrinsic-категории. Отсутствие
//! kernel не означает отсутствия самого registered owner-а. Полный concrete
//! End, включая визуальные хвосты без активного исполнения, ещё не подключён:
//! перенос ресурсов сам по себе не реализует StopAllSkills.
//! StopAllSkills (0x004CDF50) вызывает End(0) каждого экземпляра в порядке
//! attack → defense → summon → state, не очищая AI target/FIFO/background.
//! Полный registered-skill End ещё не подключён; завершение одного текущего
//! cast не заменяет этот контракт, в том числе перед приручением монстра.
//! В Luvinia Application/MoveShape.cpp AddSkill/DelSkill остались пустыми,
//! а StopAllSkills работает с другой active-module map и удаляет её записи.
//! Старый GetSkill сохранился в отключённом OtherMessage; world factory
//! хранит конфигурации. Эти формы не заменяют четыре owner-вектора Miracle.
//! Доказательства этих и остальных недостигнутых методов сохранены ниже.

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use indexmap::IndexSet;

use super::ai::baseai::CBaseAI;
use super::chbystate::{CHANGE_BODY_STATE_ID, ChangeBodyMutation, ChangeBodyState};
use super::exstate::{
    EX_STATE_ID, EX_STATE_NEW_ID, ExtendedState, ExtendedStateKind, ExtendedStateMutation,
};
use super::legacycodec::{LegacyReader, LegacyWriter};
use super::particularstate::{PARTICULAR_STATE_BYTES, PARTICULAR_STATE_ID, ParticularState};
use super::region::{CRegion, RegionCellAccessBlock};
use super::ridestate::{RIDE_STATE_ID, RideState};
use super::restorestate::{ConsumableRestoreMutation, ConsumableRestoreStateStorage};
use super::restorehpstate::{RESTORE_HP_STATE_BYTES, RESTORE_HP_STATE_ID};
use super::restorempstate::{RESTORE_MP_STATE_BYTES, RESTORE_MP_STATE_ID};
use super::scriptstate::ScriptMoveState;
use super::serverregion::{CServerRegion, RegionMembershipBlock};
use super::skills::kernel::{BattleFairyExecution, PlayerSkillExecution, SkillLifecycle, SkillTermination};
use super::teamstate::{CTeamState, TEAM_STATE_ID};
use super::shape::{
    CShape, SHAPE_CHANGE_AREA, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapePositionDispatch, ShapeResolver,
};
use crate::gameserver::appserver::skills::agilitystate::{
    AgilityState, PersistentAgilityFamilyState, PERSISTENT_AGILITY_FAMILY_STATE_BYTES,
};
use crate::gameserver::appserver::skills::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;
use crate::gameserver::appserver::skills::enlargemaxhp::ENLARGE_MAX_HP_SKILL_ID;
use crate::gameserver::appserver::skills::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;
use crate::gameserver::appserver::skills::origin::ORIGIN_SKILL_ID;
use crate::gameserver::appserver::skills::swordship::{
    SWORDSHIP_2_SKILL_ID, SWORDSHIP_3_SKILL_ID, SWORDSHIP_4_SKILL_ID, SWORDSHIP_SKILL_ID,
};
use crate::gameserver::appserver::skills::taiji::TAIJI_SKILL_ID;
use crate::gameserver::appserver::skills::wuxingearth::WUXING_EARTH_SKILL_ID;
use crate::gameserver::appserver::skills::wuxingfire::WUXING_FIRE_SKILL_ID;
use crate::gameserver::appserver::skills::wuxingmetal::WUXING_METAL_SKILL_ID;
use crate::gameserver::appserver::skills::wuxingwater::WUXING_WATER_SKILL_ID;
use crate::gameserver::appserver::skills::wuxingwood::WUXING_WOOD_SKILL_ID;
use crate::gameserver::appserver::skills::agilitystate2::{AgilityState2, AGILITY_STATE_2_BYTES};
use crate::gameserver::appserver::skills::callosity::{
    CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID,
};
use crate::gameserver::appserver::skills::callositystate::{
    CALLOSITY_STATE_BYTES, CallosityFamilyState,
};
use crate::gameserver::appserver::skills::curestate::{CureState, CURE_STATE_BYTES, CURE_STATE_SKILL_ID};
use crate::gameserver::appserver::skills::daubpoisonstate::{DAUB_POISON_STATE_BYTES, DAUB_POISON_STATE_ID, DaubPoisonState};
use crate::gameserver::appserver::skills::enlargefullmissstate::{EnlargeFullMissState, ENLARGE_FULL_MISS_STATE_BYTES};
use crate::gameserver::appserver::skills::enlargemaxhpstate::{ENLARGE_MAX_HP_STATE_BYTES, EnlargeMaxHpState};
use crate::gameserver::appserver::skills::enlargemaxmpstate::{ENLARGE_MAX_MP_STATE_BYTES, EnlargeMaxMpState};
use crate::gameserver::appserver::skills::heartenstate::{
    HeartenState, HEARTEN_STATE_BYTES,
};
use crate::gameserver::appserver::skills::heal::is_heal_skill;
use crate::gameserver::appserver::skills::healstate::{
    HEAL_STATE_BYTES, HealState,
};
use crate::gameserver::appserver::skills::furystate::{
    FURY_STATE_BYTES, FURY_STATE_SKILL_ID, FuryState,
};
use crate::gameserver::appserver::skills::ragebreakstate::{
    RAGE_BREAK_STATE_BYTES, RAGE_BREAK_STATE_ID, RageBreakState,
};
use crate::gameserver::appserver::skills::rushstate::{
    RUSH_STATE_BYTES, RUSH_STATE_ID, RushState,
};
use crate::gameserver::appserver::skills::rushstate2::{
    RUSH_2_STATE_BYTES, RUSH_2_STATE_ID, Rush2State,
};
use crate::gameserver::appserver::skills::roarstate::{
    ROAR_STATE_BYTES, ROAR_STATE_ID, RoarState,
};
use crate::gameserver::appserver::skills::energyholdingstate::{
    EnergyHoldingState, ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID,
};
use crate::gameserver::appserver::skills::lifeshieldstate::{
    LifeShieldState, LIFE_SHIELD_STATE_BYTES,
};
use crate::gameserver::appserver::skills::machineshieldstate::{
    MachineShieldState, MACHINE_SHIELD_STATE_BYTES,
};
use crate::gameserver::appserver::skills::manashieldstate::{
    ManaShieldState, MANA_SHIELD_STATE_BYTES,
};
use crate::gameserver::appserver::skills::promotionstate::{
    PromotionState, PROMOTION_STATE_BYTES,
};
use crate::gameserver::appserver::skills::knockoutstate::{
    KNOCK_OUT_STATE_BYTES, KNOCK_OUT_STATE_ID, KnockOutState,
};
use crate::gameserver::appserver::skills::boalockstate::{
    BOA_LOCK_STATE_BYTES, BOA_LOCK_STATE_ID, BoaLockState,
};
use crate::gameserver::appserver::skills::blindstate::{
    BLIND_STATE_BYTES, BLIND_STATE_ID, BlindState,
};
use crate::gameserver::appserver::skills::knightcutstate::{
    KNIGHT_CUT_STATE_BYTES, KNIGHT_CUT_STATE_ID, KnightCutState,
};
use crate::gameserver::appserver::skills::kerosenestate::{KeroseneState, KEROSENE_STATE_BYTES, KEROSENE_STATE_ID};
use crate::gameserver::appserver::skills::originstate::{ORIGIN_STATE_BYTES, OriginState};
use crate::gameserver::appserver::skills::pillarstate::{
    PILLAR_STATE_BYTES, PILLAR_STATE_ID, PillarState,
};
use crate::gameserver::appserver::skills::poisonarrowstate::{
    PoisonArrowState, POISON_ARROW_STATE_BYTES,
};
use crate::gameserver::appserver::skills::poisonfogstate::{PoisonFogState, POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID};
use crate::gameserver::appserver::skills::meteorarrowstate::{MeteorArrowState, METEOR_ARROW_MASS_SKILL_ID};
use crate::gameserver::appserver::skills::spiderpoisonstate::{SPIDER_POISON_STATE_BYTES, SpiderPoisonState};
use crate::gameserver::appserver::skills::spriteburnstate::{
    SPRITE_BURN_STATE_BYTES, SpriteBurnState,
};
use crate::gameserver::appserver::skills::spiderwebstate::{
    SPIDER_WEB_STATE_BYTES, SpiderWebState,
};
use crate::gameserver::appserver::skills::sealstate::{
    SEAL_STATE_BYTES, SEAL_STATE_ID, SealState,
};
use crate::gameserver::appserver::skills::swordshipstate::{
    SWORDSHIP_STATE_BYTES, SwordshipState,
};
use crate::gameserver::appserver::skills::strikestate::{
    STRIKE_STATE_BYTES, STRIKE_STATE_ID, StrikeState,
};
use crate::gameserver::appserver::skills::bloodlossstate::{
    BloodLossState, BLOOD_LOSS_STATE_BYTES,
};
use crate::gameserver::appserver::skills::leafcutstate::{LeafCutState, LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID};
use crate::gameserver::appserver::skills::leafcutstate2::{LeafCutState2, LEAF_CUT_2_STATE_BYTES, LEAF_CUT_2_STATE_ID};
use crate::gameserver::appserver::skills::leafcutstate3::{LeafCutState3, LEAF_CUT_3_STATE_BYTES, LEAF_CUT_3_STATE_ID};
use crate::gameserver::appserver::skills::battlefairyattributestate::{BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, BattleFairyAttributeState};
use crate::gameserver::appserver::skills::bossbluefurystate::{
    BossBlueFuryState, BossBlueFuryTick, BOSS_BLUE_FURY_STATE_BYTES,
    BOSS_BLUE_FURY_STATE_ID,
};
use crate::gameserver::appserver::skills::bossbluequakestate::{
    BossBlueQuakeState, BOSS_BLUE_QUAKE_STATE_BYTES, BOSS_BLUE_QUAKE_STATE_ID,
};
use crate::gameserver::appserver::skills::skillfactory::{CSkillFactory, SkillCategory, SkillOwner};
use crate::gameserver::appserver::skills::statefactory::known_state_record_offsets;
use crate::gameserver::appserver::skills::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::skills::taijistate::{TAIJI_STATE_BYTES, TaiJiState};
use crate::gameserver::appserver::skills::tianshenxiafanstate::{
    TIAN_SHEN_XIA_FAN_STATE_BYTES, TIAN_SHEN_XIA_FAN_STATE_ID, TianShenXiaFanState,
};
use crate::gameserver::appserver::skills::weakstate::{
    WEAK_STATE_BYTES, WEAK_STATE_ID, WeakState,
};
use crate::gameserver::appserver::skills::wangshengstate::{
    WANGSHENG_STATE_BYTES, WANGSHENG_STATE_ID, WangshengState,
};
use crate::gameserver::appserver::skills::wuxingstate::{WuXingState, WUXING_STATE_BYTES};
use crate::gameserver::appserver::skills::godblessstate::{
    GOD_BLESS_STATE_BYTES, GOD_BLESS_STATE_ID, GodBlessState,
};
use crate::gameserver::appserver::skills::godblessstate2::GOD_BLESS_STATE_2_ID;
use crate::gameserver::appserver::skills::soulcollectstate::{
    SOUL_COLLECT_STATE_BYTES, SOUL_COLLECT_STATE_ID, SoulCollectState,
};
use crate::gameserver::appserver::states::automaticrestore::{
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreState, is_automatic_restore_state_id,
};
use crate::gameserver::appserver::states::state::{
    default_additional_data, default_client_state_time,
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

const fn is_auto_start_state_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        ENLARGE_FULL_MISS_SKILL_ID
            | ENLARGE_MAX_HP_SKILL_ID
            | ENLARGE_MAX_MP_SKILL_ID
            | ORIGIN_SKILL_ID
            | SWORDSHIP_SKILL_ID
            | SWORDSHIP_2_SKILL_ID
            | SWORDSHIP_3_SKILL_ID
            | SWORDSHIP_4_SKILL_ID
            | TAIJI_SKILL_ID
            | WUXING_METAL_SKILL_ID
            | WUXING_WOOD_SKILL_ID
            | WUXING_WATER_SKILL_ID
            | WUXING_FIRE_SKILL_ID
            | WUXING_EARTH_SKILL_ID
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImmediateSkillLifecycle {
    Unbegun,
    Begun,
    Ended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RegisteredSkillExecution {
    Inactive(SkillLifecycle),
    Player(PlayerSkillExecution),
    BattleFairy(BattleFairyExecution),
    Monster(super::monster::MonsterSkillExecution),
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

/// Достигнутая common-проекция `CSkill`: identity, level и concrete owner.
/// Алгоритмы concrete attack/defense/state/summon остаются у skill owners;
/// исполнение, его ресурсы и reuse принадлежат каждому экземпляру.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapeSkill {
    id: u32,
    level: i32,
    owner: SkillOwner,
    item_position: i32,
    immediate_lifecycle: ImmediateSkillLifecycle,
    execution: RegisteredSkillExecution,
    last_used_ms: u32,
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
    pub(crate) disappear_after_dead: bool,
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

    pub(crate) fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        let deadline = self.started_ms.wrapping_add(self.keep_time_ms);
        if self.keep_time_ms == 0 || deadline <= now_ms {
            0
        } else {
            deadline.wrapping_sub(now_ms)
        }
    }

    fn from_factory(state_id: u32, factory: &CSkillFactory, now_ms: u32) -> Option<Self> {
        let properties =
            factory.query_skill_base_properties(SKILL_NOT_DISAPPEAR_AFTER_DEAD, state_id as i32)?;
        let p = |usage| properties.query_property(usage);
        Some(Self {
            state_id,
            state_type: p(SKILL_USAGE_CONST) as u16,
            keep_time_ms: p(SKILL_USAGE_STATE_PERSIST_TIME),
            started_ms: now_ms,
            last_item_tick_ms: now_ms,
            disappear_after_dead: p(80_001) != 0,
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

    fn decode_all(payload: &[u8], now_ms: u32) -> Vec<Self> {
        let mut states = Vec::new();
        for offset in 4..payload.len().saturating_sub(3) {
            if read_u32(payload, offset) != Some(UNDEAD_STATE_ID) {
                continue;
            }
            let base = offset + 4;
            if base + UNDEAD_STATE_PARAMETER_BYTES > payload.len() {
                continue;
            }
            let state_id = read_u32(payload, base + 4).unwrap_or_default();
            if state_id == 0 {
                continue;
            }
            states.push(Self {
                state_id,
                state_type: read_u16(payload, base).unwrap_or_default(),
                keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
                started_ms: now_ms,
                last_item_tick_ms: now_ms,
                disappear_after_dead: payload[base + 12] != 0,
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
            });
        }
        states
    }

    fn write_serialized(&mut self, payload: &mut [u8], offset: usize) {
        let base = offset + 4;
        write_u32(payload, offset, UNDEAD_STATE_ID);
        write_u16(payload, base, self.state_type);
        write_u32(payload, base + 4, self.state_id);
        write_u32(payload, base + 8, self.keep_time_ms);
        payload[base + 12] = u8::from(self.disappear_after_dead);
        payload[base + 13] = u8::from(self.percentage);
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
        self.serialized_offset = Some(offset);
    }

    fn update_serialized_runtime(&self, payload: &mut [u8], now_ms: u32) {
        if let Some(offset) = self.serialized_offset
            && offset + 4 + UNDEAD_STATE_PARAMETER_BYTES <= payload.len()
        {
            write_u32(payload, offset + 12, self.remaining_time_ms(now_ms));
        }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UndeadStateMutation {
    pub(crate) removed: Vec<UndeadState>,
    pub(crate) added: Option<UndeadState>,
    pub(crate) legacy_return: u32,
    pub(crate) state_list_changed: bool,
}

impl MoveShapeSkill {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMoveShape {
    shape: CShape,
    skills: [Vec<MoveShapeSkill>; 4],
    current_skill_id: Option<u32>,
    item_skill_ids: Vec<u32>,
    state_storage: CanonicalStateStorage,
    moveable_count: i32,
    moveable: bool,
    can_fight_count: i32,
    can_fight: bool,
    is_god: bool,
    pets: Vec<MoveShapePet>,
    current_pets_mode: i32,
    stiffen_started_ms: u32,
    stiffen_count: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LegacyStateCodec {
    payload: Vec<u8>,
}

impl LegacyStateCodec {
    fn replace(&mut self, payload: Vec<u8>) {
        self.payload = payload;
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CanonicalStateStorage {
    persistent_agility_family_state: Option<PersistentAgilityFamilyState>,
    agility_state_2: Option<AgilityState2>,
    callosity_state: Option<CallosityFamilyState>,
    taiji_state: Option<TaiJiState>,
    enlarge_full_miss_state: Option<EnlargeFullMissState>,
    enlarge_max_hp_state: Option<EnlargeMaxHpState>,
    enlarge_max_mp_state: Option<EnlargeMaxMpState>,
    origin_state: Option<OriginState>,
    hearten_state: Option<HeartenState>,
    heal_states: Vec<HealState>,
    fury_states: Vec<FuryState>,
    rage_break_state: Option<RageBreakState>,
    boss_blue_fury_state: Option<BossBlueFuryState>,
    boss_blue_quake_state: Option<BossBlueQuakeState>,
    cure_states: Vec<CureState>,
    daub_poison_state: Option<DaubPoisonState>,
    seal_state: Option<SealState>,
    curable_state_order: IndexSet<u32>,
    poison_arrow_state: Option<PoisonArrowState>,
    poison_fog_state: Option<PoisonFogState>,
    meteor_arrow_state: Option<MeteorArrowState>,
    spider_poison_state: Option<SpiderPoisonState>,
    sprite_burn_state: Option<SpriteBurnState>,
    spider_web_state: Option<SpiderWebState>,
    weak_state: Option<WeakState>,
    god_bless_state: Option<GodBlessState>,
    soul_collect_state: Option<SoulCollectState>,
    knock_out_state: Option<KnockOutState>,
    blind_state: Option<BlindState>,
    boa_lock_state: Option<BoaLockState>,
    rush_state: Option<RushState>,
    rush_2_state: Option<Rush2State>,
    roar_state: Option<RoarState>,
    energy_holding_state: Option<EnergyHoldingState>,
    pillar_state: Option<PillarState>,
    knight_cut_state: Option<KnightCutState>,
    blind_state_order: IndexSet<u32>,
    blood_loss_state: Option<BloodLossState>,
    leaf_cut_state: Option<LeafCutState>,
    leaf_cut_2_state: Option<LeafCutState2>,
    leaf_cut_3_state: Option<LeafCutState3>,
    kerosene_state: Option<KeroseneState>,
    swordship_states: Vec<SwordshipState>,
    strike_states: Vec<StrikeState>,
    wuxing_states: Vec<WuXingState>,
    automatic_restore_states: Vec<AutomaticRestoreState>,
    consumable_restore_states: ConsumableRestoreStateStorage,
    particular_states: Vec<ParticularState>,
    team_recruitment_states: Vec<CTeamState>,
    battle_fairy_attribute_states: Vec<BattleFairyAttributeState>,
    tian_shen_xia_fan_state: Option<TianShenXiaFanState>,
    wangsheng_state: Option<WangshengState>,
    periodic_attack_order: IndexSet<u32>,
    defense_shields: Vec<DefenseShieldState>,
    ex_states: LegacyStateCodec,
    change_body_states: Vec<ChangeBodyState>,
    extended_states: Vec<ExtendedState>,
    undead_states: Vec<UndeadState>,
    script_states: Vec<ScriptMoveState>,
    ride_state: Option<RideState>,
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

/// Типизированные состояния, влияющие на `CPlayer::UpdateProperty`.
/// Значения копируются из канонического хранилища в порядке соответствующих
/// записей `m_vStates`; сырой payload остаётся только владельцем порядка.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerPropertyState {
    PersistentAgility(PersistentAgilityFamilyState),
    Agility2(AgilityState2),
    TaiJi(TaiJiState),
    EnlargeMaxHp(EnlargeMaxHpState),
    EnlargeMaxMp(EnlargeMaxMpState),
    EnlargeFullMiss(EnlargeFullMissState),
    Origin(OriginState),
    Hearten(HeartenState),
    Callosity(CallosityFamilyState),
    Swordship(SwordshipState),
    WuXing(WuXingState),
    BattleFairyAttribute(BattleFairyAttributeState),
    TianShenXiaFan(TianShenXiaFanState),
    Weak(WeakState),
    PoisonFog(PoisonFogState),
    GodBless(GodBlessState),
    Roar(RoarState),
    Script(ScriptMoveState),
    Undead(UndeadState),
    Extended(ExtendedState),
    ChangeBody(ChangeBodyState),
    Ride(RideState),
    RageBreak(RageBreakState),
    Fury(FuryState),
    Wangsheng(WangshengState),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MonsterPropertyState {
    TaiJi(TaiJiState),
    Origin(OriginState),
    Swordship(SwordshipState),
    BattleFairyAttribute(BattleFairyAttributeState),
    Fury(FuryState),
    Weak(WeakState),
    PoisonFog(PoisonFogState),
    GodBless(GodBlessState),
    Roar(RoarState),
    BossBlueFury(BossBlueFuryState),
}

fn next_ordered_cloned_state<T: Clone>(
    states: &[T],
    state_id: u32,
    occurrences: &mut BTreeMap<u32, usize>,
    id: impl Fn(&T) -> u32,
) -> Option<T> {
    let occurrence = occurrences.entry(state_id).or_default();
    let state = states
        .iter()
        .filter(|state| id(state) == state_id)
        .nth(*occurrence)?
        .clone();
    *occurrence = occurrence.wrapping_add(1);
    Some(state)
}

fn next_ordered_state<T: Copy>(
    states: &[T],
    state_id: u32,
    occurrences: &mut BTreeMap<u32, usize>,
    id: impl Fn(T) -> u32,
) -> Option<T> {
    let occurrence = occurrences.entry(state_id).or_default();
    let state = states
        .iter()
        .copied()
        .filter(|state| id(*state) == state_id)
        .nth(*occurrence)?;
    *occurrence = occurrence.wrapping_add(1);
    Some(state)
}

impl Default for CMoveShape {
    fn default() -> Self {
        Self {
            shape: CShape::default(),
            skills: Default::default(),
            current_skill_id: None,
            item_skill_ids: Vec::new(),
            state_storage: CanonicalStateStorage::default(),
            moveable_count: 0,
            moveable: true,
            can_fight_count: 0,
            can_fight: true,
            is_god: false,
            pets: Vec::new(),
            current_pets_mode: 1,
            stiffen_started_ms: 0,
            stiffen_count: 0,
        }
    }
}

impl CMoveShape {
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

    /// Материализует общий `CMoveShape::AddToByteArray_ForClient` для всех
    /// распознанных canonical state records. Неизвестный record не позволяет
    /// доказать следующий offset, поэтому serializer возвращает `None`, а не
    /// публикует неверный count или сдвинутые поля.
    pub(crate) fn encode_client_snapshot(
        &self,
        include_child: bool,
        is_dead: bool,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.encode_client_snapshot_with_team_count(
            include_child,
            is_dead,
            now_ms,
            1,
            timed_state_now_milliseconds,
        )
    }

    /// Player-owner передаёт сюда канонический размер своей `CTeam`: точный
    /// `CTeamState::GetAdditionalData` запрашивает team session при каждом
    /// полном снимке, а при её отсутствии оставляет исходный fallback `1`.
    pub(crate) fn encode_client_snapshot_with_team_count(
        &self,
        include_child: bool,
        is_dead: bool,
        now_ms: u32,
        team_member_count: usize,
        mut timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let states = self.serialized_ex_states(now_ms, &mut timed_state_now_milliseconds);
        let declared_count = if states.is_empty() {
            0usize
        } else {
            usize::try_from(read_u32(&states, 0)?).ok()?
        };
        let offsets = known_state_record_offsets(&states);
        if offsets.len() != declared_count {
            return None;
        }
        let total_count = declared_count;
        let mut payload = Vec::new();
        self.shape
            .add_to_byte_array(&mut payload, include_child)
            .then_some(())?;
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u8(u8::from(is_dead));
        writer.write_i32(i32::try_from(total_count).ok()?);
        let mut restore_index = 0usize;
        let mut particular_index = 0usize;
        let mut team_index = 0usize;
        for offset in offsets {
            let state_id = read_i32(&states, offset)?;
            if state_id == RESTORE_HP_STATE_ID || state_id == RESTORE_MP_STATE_ID {
                let (typed_state_id, client_time) = self
                    .consumable_restore_states
                    .client_snapshot_record(restore_index, &mut timed_state_now_milliseconds)?;
                if typed_state_id != state_id {
                    return None;
                }
                restore_index += 1;
                writer.write_i32(state_id);
                writer.write_i32(client_time);
                writer.write_u32(default_additional_data());
                continue;
            }
            if state_id == PARTICULAR_STATE_ID as i32 {
                let state = self.particular_states.get(particular_index)?;
                if read_u32(&states, offset + 4) != Some(state.additional_data()) {
                    return None;
                }
                particular_index += 1;
                writer.write_i32(state_id);
                writer.write_i32(state.client_state_time());
                writer.write_u32(state.additional_data());
                continue;
            }
            if state_id == TEAM_STATE_ID {
                let state = self.team_recruitment_states.get(team_index)?;
                team_index += 1;
                writer.write_i32(state_id);
                writer.write_i32(state.client_state_time());
                writer.write_u32(state.additional_data(team_member_count));
                writer.write_c_string(state.team_name());
                continue;
            }
            writer.write_i32(state_id);
            writer.write_i32(self.client_state_time(
                &states,
                offset,
                state_id as u32,
                now_ms,
                &mut timed_state_now_milliseconds,
            )?);
            writer.write_u32(self.client_state_additional_data(state_id as u32));
        }
        if restore_index != self.consumable_restore_states.len() {
            return None;
        }
        if particular_index != self.particular_states.len()
            || team_index != self.team_recruitment_states.len()
        {
            return None;
        }
        Some(payload)
    }

    /// Exact virtual `CState::GetClientStateTime`: место persisted remaining
    /// зависит от concrete serializer-а, а у постоянных состояний второй
    /// DWORD вообще является игровым параметром. Особые owner-ы читаются из
    /// канонического состояния; только записи с доказанным `remaining` сразу
    /// после ID используют общий codec.
    fn client_state_time(
        &self,
        states: &[u8],
        offset: usize,
        state_id: u32,
        now_ms: u32,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Option<i32> {
        let permanent = matches!(
            state_id,
            SWORDSHIP_SKILL_ID
                | SWORDSHIP_2_SKILL_ID
                | SWORDSHIP_3_SKILL_ID
                | SWORDSHIP_4_SKILL_ID
                | WUXING_METAL_SKILL_ID
                | WUXING_WOOD_SKILL_ID
                | WUXING_WATER_SKILL_ID
                | WUXING_FIRE_SKILL_ID
                | WUXING_EARTH_SKILL_ID
                | METEOR_ARROW_MASS_SKILL_ID
                | WEAK_STATE_ID
                | ENERGY_HOLDING_STATE_ID
                | CURE_STATE_SKILL_ID
                | ENLARGE_FULL_MISS_SKILL_ID
                | TAIJI_SKILL_ID
                | ENLARGE_MAX_HP_SKILL_ID
                | ENLARGE_MAX_MP_SKILL_ID
                | ORIGIN_SKILL_ID
                | RIDE_STATE_ID
        ) || PersistentAgilityFamilyState::is_known_skill(state_id)
            || is_automatic_restore_state_id(state_id);
        if permanent {
            return Some(default_client_state_time());
        }
        match state_id {
            SOUL_COLLECT_STATE_ID => Some(
                self.soul_collect_state
                    .map_or(default_client_state_time(), |state| state.variable_percent() as i32),
            ),
            CHANGE_BODY_STATE_ID => self
                .change_body_states
                .iter()
                .find(|state| state.serialized_span().is_some_and(|(start, _)| start == offset))
                .map(|state| state.remaining_time_ms(now_ms) as i32),
            EX_STATE_ID | EX_STATE_NEW_ID => self
                .extended_states
                .iter()
                .find(|state| state.serialized_span().is_some_and(|(start, _)| start == offset))
                .map(|state| state.remaining_time_ms(now_ms) as i32),
            UNDEAD_STATE_ID => self
                .undead_states
                .iter()
                .find(|state| state.serialized_span().is_some_and(|(start, _)| start == offset))
                .map(|state| state.remaining_time_ms(now_ms) as i32),
            LEAF_CUT_STATE_ID => self
                .leaf_cut_state
                .map(|state| state.client_state_time(&mut now_milliseconds) as i32),
            LEAF_CUT_2_STATE_ID => self
                .leaf_cut_2_state
                .map(|state| state.client_state_time(&mut now_milliseconds) as i32),
            LEAF_CUT_3_STATE_ID => self
                .leaf_cut_3_state
                .map(|state| state.client_state_time(&mut now_milliseconds) as i32),
            POISON_FOG_STATE_ID => self
                .poison_fog_state
                .map(|state| state.client_time(&mut now_milliseconds)),
            id if id == super::skills::spriteburn::SPRITE_BURN_SKILL_ID => self
                .sprite_burn_state
                .map(|state| state.client_state_time(&mut now_milliseconds) as i32),
            id if id == super::skills::spiderpoison::SPIDER_POISON_SKILL_ID => self
                .spider_poison_state
                .map(|state| state.client_state_time(&mut now_milliseconds) as i32),
            id if id == super::skills::bloodloss::BLOOD_LOSS_SKILL_ID => self
                .blood_loss_state
                .map(|state| state.client_state_time(&mut now_milliseconds) as i32),
            id if id == super::skills::poisonarrow::POISON_ARROW_SKILL_ID => self
                .poison_arrow_state
                .map(|state| state.client_state_time(&mut now_milliseconds) as i32),
            _ => read_i32(states, offset + 4),
        }
    }

    /// Exact virtual `CState::GetAdditionalData`: persisted tail не является
    /// client-проекцией. Override-ы берутся из соответствующего typed owner-а;
    /// остальные состояния используют нулевую базовую реализацию.
    fn client_state_additional_data(&self, state_id: u32) -> u32 {
        match state_id {
            WEAK_STATE_ID => self
                .weak_state
                .map_or(default_additional_data(), WeakState::attack_loss),
            SOUL_COLLECT_STATE_ID => self
                .soul_collect_state
                .map_or(default_additional_data(), |state| state.souls() as u32),
            ENERGY_HOLDING_STATE_ID => self
                .energy_holding_state
                .map_or(default_additional_data(), EnergyHoldingState::parameter_percent),
            METEOR_ARROW_MASS_SKILL_ID => self
                .meteor_arrow_state
                .map_or(default_additional_data(), |state| state.additional_data() as u32),
            RIDE_STATE_ID => self
                .ride_state
                .as_ref()
                .map_or(default_additional_data(), RideState::additional_data),
            id if matches!(
                id,
                super::skills::machineshield::MACHINE_SHIELD_SKILL_ID
                    | super::skills::manashield::MANA_SHIELD_SKILL_ID
                    | super::skills::lifeshield::LIFE_SHIELD_SKILL_ID
            ) => self
                .defense_shields
                .iter()
                .find(|state| state.skill_id() == id)
                .map_or(default_additional_data(), |state| match state {
                    DefenseShieldState::Life(state) => state.life() as u32,
                    DefenseShieldState::Machine(state) => state.life() as u32,
                    DefenseShieldState::Mana(state) => state.life() as u32,
                    DefenseShieldState::Promotion(_) => default_additional_data(),
                }),
            _ => default_additional_data(),
        }
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

    pub(crate) fn skills_in_category(&self, category: SkillCategory) -> &[MoveShapeSkill] {
        &self.skills[category as usize]
    }

    /// `AutoStartPassiveSkill`: state-вектор обходится в порядке
    /// вставки, а каждый `IsAutoStart != 0` добавляется в background-очередь.
    /// Self-target `Begin(this, this)` в Rust задаётся самим владельцем.
    pub(crate) fn auto_start_passive_skills(&mut self, ai: &mut CBaseAI) -> usize {
        let mut count = 0;
        for skill in &mut self.skills[SkillCategory::State as usize] {
            if is_auto_start_state_skill(skill.id) {
                skill.immediate_lifecycle = ImmediateSkillLifecycle::Begun;
                ai.add_pending_back_stage_skill(skill.id);
                count += 1;
            }
        }
        count
    }

    pub(crate) fn immediate_skill_ended(&self, skill_id: u32, factory: &CSkillFactory) -> bool {
        self.skill(skill_id, factory).is_some_and(|skill| skill.immediate_lifecycle != ImmediateSkillLifecycle::Begun)
    }

    pub(crate) fn begin_immediate_skill(&mut self, skill_id: u32, factory: &CSkillFactory) {
        if let Some(skill) = self.skill_mut(skill_id, factory) {
            skill.immediate_lifecycle = ImmediateSkillLifecycle::Begun;
        }
    }

    pub(crate) fn immediate_skill_started(&self, skill_id: u32, factory: &CSkillFactory) -> bool {
        self.skill(skill_id, factory).is_some_and(|skill| skill.immediate_lifecycle == ImmediateSkillLifecycle::Begun)
    }

    pub(crate) fn finish_immediate_skill(&mut self, skill_id: u32, factory: &CSkillFactory) {
        if let Some(skill) = self.skill_mut(skill_id, factory) {
            skill.immediate_lifecycle = ImmediateSkillLifecycle::Ended;
            skill.execution.lifecycle_mut().reset_after_end(SkillTermination::Completed);
        }
    }

    pub(crate) fn undead_states(&self) -> &[UndeadState] {
        &self.undead_states
    }

    pub(crate) fn serialize_ex_states_for_save(
        &mut self,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Vec<u8> {
        let payload = self.serialized_ex_states(now_ms, timed_state_now_milliseconds);
        for state in &mut self.extended_states {
            state.commit_saved_time(now_ms);
        }
        for state in &mut self.change_body_states {
            state.commit_saved_time(now_ms);
        }
        for state in &mut self.undead_states {
            state.keep_time_ms = state.remaining_time_ms(now_ms);
        }
        payload
    }

    pub(crate) fn serialized_ex_states(
        &self,
        now_ms: u32,
        mut timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Vec<u8> {
        let mut payload = self.ex_states.to_vec();
        for state in &self.change_body_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        for state in &self.extended_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        for state in &self.undead_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        let mut script_occurrences = BTreeMap::<i32, usize>::new();
        for state in &self.script_states {
            let state_id = state.state_id();
            let occurrence = script_occurrences.entry(state_id).or_default();
            let offset = known_state_record_offsets(&payload)
                .into_iter()
                .filter(|offset| read_u32(&payload, *offset) == Some(state_id as u32))
                .nth(*occurrence);
            *occurrence += 1;
            let record = state.encoded(&mut timed_state_now_milliseconds);
            if let Some(offset) = offset
                && let Some(destination) = payload.get_mut(offset..offset + record.len())
            {
                destination.copy_from_slice(&record);
            }
        }
        let mut restore_occurrences = BTreeMap::<i32, usize>::new();
        for index in 0..self.consumable_restore_states.len() {
            let Some(state_id) = self.consumable_restore_states.state_id(index) else {
                continue;
            };
            let occurrence = restore_occurrences.entry(state_id).or_default();
            let offset = known_state_record_offsets(&payload)
                .into_iter()
                .filter(|offset| read_i32(&payload, *offset) == Some(state_id))
                .nth(*occurrence);
            *occurrence += 1;
            let Some(record) = self
                .consumable_restore_states
                .persisted_record(index, &mut timed_state_now_milliseconds)
            else {
                continue;
            };
            if let Some(offset) = offset
                && let Some(destination) = payload.get_mut(offset..offset + record.len())
            {
                destination.copy_from_slice(&record);
            }
        }
        if let Some(state) = self.leaf_cut_state {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        if let Some(state) = self.leaf_cut_2_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        if let Some(state) = self.leaf_cut_3_state {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        if let Some(state) = self.kerosene_state { state.update_serialized_runtime(&mut payload, now_ms); }
        if let Some(state) = self.poison_fog_state { state.update_serialized_runtime(&mut payload, now_ms); }
        if let Some(state) = self.meteor_arrow_state { state.update_serialized(&mut payload); }
        if let Some(state) = self.blind_state {
            if let Some(offset) = known_state_record_offsets(&payload)
                .into_iter()
                .find(|offset| read_u32(&payload, *offset) == Some(BLIND_STATE_ID))
            {
                write_u32(
                    &mut payload,
                    offset + 4,
                    state.client_state_time(&mut timed_state_now_milliseconds),
                );
            }
        }
        if let Some(state) = self.seal_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &[
                    state.skill_id().to_le_bytes(),
                    (state.client_time(&mut timed_state_now_milliseconds) as u32).to_le_bytes(),
                ]
                .concat(),
            );
        }
        let strike_offsets: Vec<_> = known_state_record_offsets(&payload)
            .into_iter()
            .filter(|offset| read_u32(&payload, *offset) == Some(STRIKE_STATE_ID))
            .collect();
        for (state, offset) in self.strike_states.iter().zip(strike_offsets) {
            write_u32(
                &mut payload,
                offset + 4,
                state.client_time(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.knock_out_state {
            if let Some(offset) = known_state_record_offsets(&payload)
                .into_iter()
                .find(|offset| read_u32(&payload, *offset) == Some(KNOCK_OUT_STATE_ID))
            {
                write_u32(
                    &mut payload,
                    offset + 4,
                    state.client_time(&mut timed_state_now_milliseconds) as u32,
                );
            }
        }
        if let Some(state) = self.spider_web_state {
            if let Some(offset) = known_state_record_offsets(&payload)
                .into_iter()
                .find(|offset| read_u32(&payload, *offset) == Some(state.skill_id()))
            {
                write_u32(
                    &mut payload,
                    offset + 4,
                    state.client_time(&mut timed_state_now_milliseconds) as u32,
                );
            }
        }
        if let Some(state) = self.god_bless_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.soul_collect_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded());
        }
        if let Some(state) = self.sprite_burn_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(&mut timed_state_now_milliseconds));
        }
        if let Some(state) = self.spider_poison_state { update_known_state_record(&mut payload, state.skill_id(), &state.encoded(&mut timed_state_now_milliseconds)); }
        if let Some(state) = self.daub_poison_state { update_known_state_record(&mut payload, state.skill_id(), &state.encoded(&mut timed_state_now_milliseconds)); }
        if let Some(state) = self.boss_blue_quake_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.knight_cut_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.boa_lock_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.rush_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.rush_2_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        if let Some(state) = self.roar_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.pillar_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.rage_break_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.hearten_state {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        for state in &self.heal_states {
            update_known_state_record(
                &mut payload,
                state.skill_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        let fury_offsets: Vec<_> = known_state_record_offsets(&payload)
            .into_iter()
            .filter(|offset| read_u32(&payload, *offset) == Some(FURY_STATE_SKILL_ID))
            .collect();
        for (state, offset) in self.fury_states.iter().zip(fury_offsets) {
            let record = state.encoded(&mut timed_state_now_milliseconds);
            if let Some(destination) = payload.get_mut(offset..offset + FURY_STATE_BYTES) {
                destination.copy_from_slice(&record);
            }
        }
        if let Some(state) = self.agility_state_2 {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        if let Some(state) = self.blood_loss_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        if let Some(state) = self.energy_holding_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded());
        }
        if let Some(state) = self.callosity_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        if let Some(state) = self.boss_blue_fury_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        if let Some(state) = self.poison_arrow_state {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        for state in &self.defense_shields {
            match state {
                DefenseShieldState::Mana(state) => {
                    update_known_state_record(
                        &mut payload,
                        state.skill_id(),
                        &state.encoded(&mut timed_state_now_milliseconds),
                    );
                }
                DefenseShieldState::Machine(state) => {
                    update_known_state_record(
                        &mut payload,
                        state.skill_id(),
                        &state.encoded(&mut timed_state_now_milliseconds),
                    );
                }
                DefenseShieldState::Life(state) => {
                    update_known_state_record(
                        &mut payload,
                        state.skill_id(),
                        &state.encoded(&mut timed_state_now_milliseconds),
                    );
                }
                DefenseShieldState::Promotion(state) => {
                    update_known_state_record(
                        &mut payload,
                        state.skill_id(),
                        &state.encoded(&mut timed_state_now_milliseconds),
                    );
                }
            }
        }
        for state in &self.battle_fairy_attribute_states {
            update_known_state_record(&mut payload, state.skill_id(), &state.encoded(now_ms));
        }
        if let Some(state) = self.tian_shen_xia_fan_state {
            update_known_state_record(
                &mut payload,
                state.state_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        if let Some(state) = self.wangsheng_state {
            update_known_state_record(
                &mut payload,
                state.state_id(),
                &state.encoded(&mut timed_state_now_milliseconds),
            );
        }
        payload
    }

    pub(crate) fn replace_ex_states(&mut self, states: Vec<u8>, skill_factory: &CSkillFactory) {
        let known_offsets = known_state_record_offsets(&states);
        let state_owner = self.shape.identity();
        self.consumable_restore_states =
            ConsumableRestoreStateStorage::decode_known(&states, &known_offsets);
        self.automatic_restore_states = known_offsets
            .iter()
            .copied()
            .filter_map(|offset| AutomaticRestoreState::decode(&states, offset))
            .collect();
        self.particular_states = known_offsets
            .iter()
            .copied()
            .filter(|offset| read_u32(&states, *offset) == Some(PARTICULAR_STATE_ID))
            .filter_map(|offset| ParticularState::decode(&states, offset).ok())
            .collect();
        self.team_recruitment_states = known_offsets
            .iter()
            .copied()
            .filter(|offset| read_i32(&states, *offset) == Some(TEAM_STATE_ID))
            .filter_map(|offset| CTeamState::decode(&states, offset).ok())
            .collect();
        self.change_body_states = ChangeBodyState::decode_all(&states, 0);
        self.change_body_states.retain(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.extended_states = ExtendedState::decode_all(&states, 0);
        self.extended_states.retain(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.undead_states = UndeadState::decode_all(&states, 0);
        self.undead_states.retain(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.ride_state = RideState::decode(&states).filter(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.script_states = known_offsets
            .iter()
            .copied()
            .filter_map(|offset| ScriptMoveState::decode(&states, offset).ok())
            .collect();
        self.periodic_attack_order.shift_remove(&LEAF_CUT_STATE_ID);
        self.leaf_cut_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(LEAF_CUT_STATE_ID))
            .and_then(|offset| LeafCutState::decode(&states, offset, 0).ok());
        if let Some(state) = self.leaf_cut_state {
            self.periodic_attack_order.insert(state.skill_id());
        }
        self.periodic_attack_order.shift_remove(&LEAF_CUT_2_STATE_ID);
        self.leaf_cut_2_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(LEAF_CUT_2_STATE_ID))
            .and_then(|offset| LeafCutState2::decode(&states, offset, 0).ok());
        if self.leaf_cut_2_state.is_some() { self.periodic_attack_order.insert(LEAF_CUT_2_STATE_ID); }
        self.periodic_attack_order.shift_remove(&LEAF_CUT_3_STATE_ID);
        self.leaf_cut_3_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(LEAF_CUT_3_STATE_ID))
            .and_then(|offset| LeafCutState3::decode(&states, offset, 0).ok());
        if let Some(state) = self.leaf_cut_3_state {
            self.periodic_attack_order.insert(state.skill_id());
        }
        self.periodic_attack_order.shift_remove(&KEROSENE_STATE_ID);
        self.kerosene_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(KEROSENE_STATE_ID)).and_then(|offset| KeroseneState::decode(&states, offset, 0).ok());
        if let Some(state) = self.kerosene_state { self.periodic_attack_order.insert(state.skill_id()); }
        self.curable_state_order.shift_remove(&POISON_FOG_STATE_ID);
        self.poison_fog_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(POISON_FOG_STATE_ID)).and_then(|offset| PoisonFogState::decode(&states, offset, 0).ok());
        if let Some(state) = self.poison_fog_state { self.curable_state_order.insert(state.skill_id()); }
        self.meteor_arrow_state = known_offsets.iter().copied()
            .find(|offset| read_u32(&states, *offset) == Some(METEOR_ARROW_MASS_SKILL_ID))
            .and_then(|offset| MeteorArrowState::decode(&states, offset).ok());
        self.blind_state_order.shift_remove(&BLIND_STATE_ID);
        self.blind_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(BLIND_STATE_ID))
            .and_then(|offset| BlindState::decode(&states, offset).ok());
        if let Some(state) = self.blind_state {
            self.blind_state_order.insert(state.skill_id());
        }
        self.blind_state_order.shift_remove(&KNOCK_OUT_STATE_ID);
        self.curable_state_order.shift_remove(&KNOCK_OUT_STATE_ID);
        self.knock_out_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(KNOCK_OUT_STATE_ID))
            .and_then(|offset| KnockOutState::decode(&states, offset).ok());
        if let Some(state) = self.knock_out_state {
            self.blind_state_order.insert(state.skill_id());
            self.curable_state_order.insert(state.skill_id());
        }
        self.blind_state_order.shift_remove(&super::skills::spiderweb::SPIDER_WEB_SKILL_ID);
        self.curable_state_order.shift_remove(&super::skills::spiderweb::SPIDER_WEB_SKILL_ID);
        self.spider_web_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(super::skills::spiderweb::SPIDER_WEB_SKILL_ID))
            .and_then(|offset| SpiderWebState::decode(&states, offset).ok());
        if let Some(state) = self.spider_web_state {
            self.blind_state_order.insert(state.skill_id());
            self.curable_state_order.insert(state.skill_id());
        }
        self.blind_state_order.shift_remove(&SEAL_STATE_ID);
        self.curable_state_order.shift_remove(&SEAL_STATE_ID);
        self.seal_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(SEAL_STATE_ID))
            .and_then(|offset| SealState::decode(&states, offset).ok());
        if let Some(state) = self.seal_state {
            self.blind_state_order.insert(state.skill_id());
            self.curable_state_order.insert(state.skill_id());
        }
        self.god_bless_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset).is_some_and(|id| matches!(id, GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID)))
            .and_then(|offset| GodBlessState::decode(&states, offset).ok());
        self.weak_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(WEAK_STATE_ID))
            .and_then(|offset| WeakState::decode(&states, offset).ok());
        self.soul_collect_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(SOUL_COLLECT_STATE_ID))
            .and_then(|offset| SoulCollectState::decode(&states, offset).ok());
        self.periodic_attack_order.shift_remove(&super::skills::spriteburn::SPRITE_BURN_SKILL_ID);
        self.curable_state_order.shift_remove(&super::skills::spriteburn::SPRITE_BURN_SKILL_ID);
        self.sprite_burn_state = known_offsets.iter().copied()
            .find(|offset| read_u32(&states, *offset) == Some(super::skills::spriteburn::SPRITE_BURN_SKILL_ID))
            .and_then(|offset| SpriteBurnState::decode(&states, offset, 0).ok());
        if let Some(state) = self.sprite_burn_state { self.periodic_attack_order.insert(state.skill_id()); self.curable_state_order.insert(state.skill_id()); }
        self.periodic_attack_order.shift_remove(&super::skills::spiderpoison::SPIDER_POISON_SKILL_ID); self.curable_state_order.shift_remove(&super::skills::spiderpoison::SPIDER_POISON_SKILL_ID);
        self.spider_poison_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(super::skills::spiderpoison::SPIDER_POISON_SKILL_ID)).and_then(|offset| SpiderPoisonState::decode(&states, offset, 0).ok());
        if let Some(state) = self.spider_poison_state { self.periodic_attack_order.insert(state.skill_id()); self.curable_state_order.insert(state.skill_id()); }
        self.daub_poison_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(DAUB_POISON_STATE_ID)).and_then(|offset| DaubPoisonState::decode(&states, offset).ok());
        self.curable_state_order.shift_remove(&BOSS_BLUE_QUAKE_STATE_ID);
        self.boss_blue_quake_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(BOSS_BLUE_QUAKE_STATE_ID))
            .and_then(|offset| BossBlueQuakeState::decode(&states, offset).ok());
        if self.boss_blue_quake_state.is_some() {
            self.curable_state_order.insert(BOSS_BLUE_QUAKE_STATE_ID);
        }
        self.curable_state_order.shift_remove(&KNIGHT_CUT_STATE_ID);
        self.knight_cut_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(KNIGHT_CUT_STATE_ID))
            .and_then(|offset| KnightCutState::decode(&states, offset).ok());
        if self.knight_cut_state.is_some() {
            self.curable_state_order.insert(KNIGHT_CUT_STATE_ID);
        }
        self.curable_state_order.shift_remove(&BOA_LOCK_STATE_ID);
        self.boa_lock_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(BOA_LOCK_STATE_ID))
            .and_then(|offset| BoaLockState::decode(&states, offset).ok());
        if self.boa_lock_state.is_some() {
            self.curable_state_order.insert(BOA_LOCK_STATE_ID);
        }
        self.curable_state_order.shift_remove(&RUSH_STATE_ID);
        self.rush_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(RUSH_STATE_ID))
            .and_then(|offset| RushState::decode(&states, offset).ok());
        if self.rush_state.is_some() {
            self.curable_state_order.insert(RUSH_STATE_ID);
        }
        self.curable_state_order.shift_remove(&RUSH_2_STATE_ID);
        self.rush_2_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(RUSH_2_STATE_ID))
            .and_then(|offset| Rush2State::decode(&states, offset).ok());
        if self.rush_2_state.is_some() {
            self.curable_state_order.insert(RUSH_2_STATE_ID);
        }
        self.roar_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(ROAR_STATE_ID))
            .and_then(|offset| RoarState::decode(&states, offset).ok());
        self.pillar_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(PILLAR_STATE_ID))
            .and_then(|offset| PillarState::decode(&states, offset).ok());
        self.rage_break_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(RAGE_BREAK_STATE_ID))
            .and_then(|offset| RageBreakState::decode(&states, offset).ok());
        self.swordship_states = known_offsets
            .iter()
            .copied()
            .filter(|offset| {
                read_u32(&states, *offset).is_some_and(super::skills::swordship::is_swordship_skill)
            })
            .filter_map(|offset| SwordshipState::decode(&states, offset).ok())
            .collect();
        self.blind_state_order.shift_remove(&STRIKE_STATE_ID);
        self.strike_states = known_offsets
            .iter()
            .copied()
            .filter(|offset| read_u32(&states, *offset) == Some(STRIKE_STATE_ID))
            .filter_map(|offset| StrikeState::decode(&states, offset).ok())
            .collect();
        if !self.strike_states.is_empty() {
            self.blind_state_order.insert(STRIKE_STATE_ID);
        }
        self.wuxing_states = known_offsets
            .iter()
            .copied()
            .filter_map(|offset| WuXingState::decode(&states, offset).ok())
            .collect();
        self.heal_states = known_offsets
            .iter()
            .copied()
            .filter(|offset| read_u32(&states, *offset).is_some_and(is_heal_skill))
            .filter_map(|offset| HealState::decode(&states, offset, state_owner).ok())
            .collect();
        self.fury_states = known_offsets
            .iter()
            .copied()
            .filter(|offset| read_u32(&states, *offset) == Some(FURY_STATE_SKILL_ID))
            .filter_map(|offset| FuryState::decode(&states, offset).ok())
            .collect();
        self.cure_states = known_offsets
            .iter()
            .copied()
            .filter(|offset| read_u32(&states, *offset) == Some(CURE_STATE_SKILL_ID))
            .filter_map(|offset| CureState::decode(&states, offset).ok())
            .collect();
        self.enlarge_full_miss_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(super::skills::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID))
            .and_then(|offset| EnlargeFullMissState::decode(&states, offset).ok());
        self.taiji_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(TAIJI_SKILL_ID)).and_then(|offset| TaiJiState::decode(&states, offset).ok());
        self.enlarge_max_hp_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(ENLARGE_MAX_HP_SKILL_ID)).and_then(|offset| EnlargeMaxHpState::decode(&states, offset).ok());
        self.enlarge_max_mp_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(ENLARGE_MAX_MP_SKILL_ID)).and_then(|offset| EnlargeMaxMpState::decode(&states, offset).ok());
        self.origin_state = known_offsets.iter().copied().find(|offset| read_u32(&states, *offset) == Some(ORIGIN_SKILL_ID)).and_then(|offset| OriginState::decode(&states, offset).ok());
        self.hearten_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(super::skills::hearten::HEARTEN_SKILL_ID))
            .and_then(|offset| HeartenState::decode(&states, offset, 0).ok());
        self.persistent_agility_family_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset).is_some_and(PersistentAgilityFamilyState::is_known_skill))
            .and_then(|offset| PersistentAgilityFamilyState::decode(&states, offset).ok());
        self.agility_state_2 = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(super::skills::agility2::AGILITY_2_SKILL_ID))
            .and_then(|offset| AgilityState2::decode(&states, offset, 0).ok());
        self.periodic_attack_order
            .shift_remove(&super::skills::bloodloss::BLOOD_LOSS_SKILL_ID);
        self.blood_loss_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(super::skills::bloodloss::BLOOD_LOSS_SKILL_ID))
            .and_then(|offset| BloodLossState::decode(&states, offset, 0).ok());
        if self.blood_loss_state.is_some() {
            self.periodic_attack_order.insert(super::skills::bloodloss::BLOOD_LOSS_SKILL_ID);
        }
        self.energy_holding_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(ENERGY_HOLDING_STATE_ID))
            .and_then(|offset| {
                let level = read_u32(&states, offset + 4)?;
                let parameter_percent = skill_factory
                    .query_skill_base_properties(ENERGY_HOLDING_STATE_ID, level as i32)?
                    .query_property(super::skills::energyholding::PARAMETER_PERCENT);
                EnergyHoldingState::decode(&states, offset, parameter_percent).ok()
            });
        self.callosity_state = known_offsets
            .iter()
            .copied()
            .find(|offset| {
                read_u32(&states, *offset)
                    .is_some_and(|id| matches!(id, CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID))
            })
            .and_then(|offset| CallosityFamilyState::decode(&states, offset).ok());
        self.boss_blue_fury_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(BOSS_BLUE_FURY_STATE_ID))
            .and_then(|offset| BossBlueFuryState::decode(&states, offset, 0).ok());
        self.periodic_attack_order
            .shift_remove(&super::skills::poisonarrow::POISON_ARROW_SKILL_ID);
        self.poison_arrow_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(super::skills::poisonarrow::POISON_ARROW_SKILL_ID))
            .and_then(|offset| PoisonArrowState::decode(&states, offset, 0).ok());
        if self.poison_arrow_state.is_some() {
            self.periodic_attack_order.insert(super::skills::poisonarrow::POISON_ARROW_SKILL_ID);
        }
        self.defense_shields.clear();
        self.defense_shields.extend(known_offsets.iter().copied().filter_map(|offset| {
            match read_u32(&states, offset) {
                Some(super::skills::lifeshield::LIFE_SHIELD_SKILL_ID) => {
                    LifeShieldState::decode(&states, offset, 0)
                        .ok()
                        .map(DefenseShieldState::Life)
                }
                Some(super::skills::manashield::MANA_SHIELD_SKILL_ID) => {
                    ManaShieldState::decode(&states, offset, 0)
                        .ok()
                        .map(DefenseShieldState::Mana)
                }
                Some(super::skills::machineshield::MACHINE_SHIELD_SKILL_ID) => {
                    MachineShieldState::decode(&states, offset, 0)
                        .ok()
                        .map(DefenseShieldState::Machine)
                }
                Some(super::skills::promotion::PROMOTION_SKILL_ID) => {
                    PromotionState::decode(&states, offset, 0)
                        .ok()
                        .map(DefenseShieldState::Promotion)
                }
                _ => None,
            }
        }));
        self.battle_fairy_attribute_states = known_offsets
            .iter()
            .copied()
            .filter_map(|offset| BattleFairyAttributeState::decode(&states, offset).ok())
            .collect();
        self.tian_shen_xia_fan_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(TIAN_SHEN_XIA_FAN_STATE_ID))
            .and_then(|offset| TianShenXiaFanState::decode(&states, offset).ok());
        self.wangsheng_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(WANGSHENG_STATE_ID))
            .and_then(|offset| WangshengState::decode(&states, offset).ok());
        self.ex_states.replace(states);
    }

    pub(crate) fn clear_persisted_runtime_state(&mut self) {
        self.skills.iter_mut().for_each(Vec::clear);
        self.current_skill_id = None;
        self.item_skill_ids.clear();
        self.ex_states.clear();
        self.persistent_agility_family_state = None;
        self.agility_state_2 = None;
        self.callosity_state = None;
        self.taiji_state = None;
        self.enlarge_full_miss_state = None;
        self.enlarge_max_hp_state = None;
        self.enlarge_max_mp_state = None;
        self.origin_state = None;
        self.meteor_arrow_state = None;
        self.hearten_state = None;
        self.heal_states.clear();
        self.fury_states.clear();
        self.rage_break_state = None;
        self.boss_blue_fury_state = None;
        self.boss_blue_quake_state = None;
        self.cure_states.clear();
        self.seal_state = None;
        self.curable_state_order.clear();
        self.poison_arrow_state = None;
        self.poison_fog_state = None;
        self.spider_poison_state = None;
        self.sprite_burn_state = None;
        self.spider_web_state = None;
        self.weak_state = None;
        self.god_bless_state = None;
        self.soul_collect_state = None;
        self.knock_out_state = None;
        self.blind_state = None;
        self.boa_lock_state = None;
        self.rush_state = None;
        self.rush_2_state = None;
        self.roar_state = None;
        self.energy_holding_state = None;
        self.pillar_state = None;
        self.knight_cut_state = None;
        self.blind_state_order.clear();
        self.blood_loss_state = None;
        self.leaf_cut_state = None;
        self.leaf_cut_2_state = None;
        self.leaf_cut_3_state = None;
        self.kerosene_state = None;
        self.swordship_states.clear();
        self.strike_states.clear();
        self.wuxing_states.clear();
        self.automatic_restore_states.clear();
        self.particular_states.clear();
        self.team_recruitment_states.clear();
        self.consumable_restore_states = ConsumableRestoreStateStorage::default();
        self.battle_fairy_attribute_states.clear();
        self.tian_shen_xia_fan_state = None;
        self.wangsheng_state = None;
        self.periodic_attack_order.clear();
        self.defense_shields.clear();
        self.change_body_states.clear();
        self.extended_states.clear();
        self.undead_states.clear();
        self.script_states.clear();
        self.ride_state = None;
        self.can_fight_count = 0;
        self.can_fight = true;
    }

    pub(crate) const fn ride_state(&self) -> Option<&RideState> {
        self.state_storage.ride_state.as_ref()
    }

    pub(crate) const fn ride_state_mut(&mut self) -> Option<&mut RideState> {
        self.state_storage.ride_state.as_mut()
    }

    /// Начальный scalar-prefix `CMoveShape::OnEnterRegion` и итог повторного
    /// `StartAllStates(false)`. Safe owner уже хранит применённые состояния,
    /// поэтому их aggregate lock-count переносится через native reset без
    /// повторного добавления тех же typed state объектов.
    pub(crate) const fn reset_region_entry_control(&mut self) {
        let moveable_locks = if self.moveable_count > 0 {
            self.moveable_count
        } else {
            0
        };
        let can_fight_locks = if self.can_fight_count > 0 {
            self.can_fight_count
        } else {
            0
        };
        self.moveable = true;
        self.can_fight = true;
        self.moveable_count = 0;
        self.can_fight_count = 0;
        self.moveable_count = moveable_locks;
        self.can_fight_count = can_fight_locks;
        self.moveable = self.moveable_count < 1;
        self.can_fight = self.can_fight_count < 1;
    }

    pub(crate) const fn has_ride_state(&self) -> bool {
        self.state_storage.ride_state.is_some()
    }

    pub(crate) const fn has_materialized_abnormality(&self) -> bool {
        self.state_storage.agility_state_2.is_some()
            || self.state_storage.hearten_state.is_some()
            || !self.state_storage.heal_states.is_empty()
            || !self.state_storage.fury_states.is_empty()
            || self.state_storage.rage_break_state.is_some()
            || self.state_storage.boss_blue_fury_state.is_some()
            || self.state_storage.boss_blue_quake_state.is_some()
            || !self.state_storage.cure_states.is_empty()
            || self.state_storage.daub_poison_state.is_some()
            || self.state_storage.seal_state.is_some()
            || self.state_storage.poison_arrow_state.is_some()
            || self.state_storage.poison_fog_state.is_some()
            || self.state_storage.spider_poison_state.is_some()
            || self.state_storage.sprite_burn_state.is_some()
            || self.state_storage.spider_web_state.is_some()
            || self.state_storage.weak_state.is_some()
            || self.state_storage.god_bless_state.is_some()
            || self.state_storage.soul_collect_state.is_some()
            || self.state_storage.knock_out_state.is_some()
            || self.state_storage.blind_state.is_some()
            || self.state_storage.boa_lock_state.is_some()
            || self.state_storage.rush_state.is_some()
            || self.state_storage.rush_2_state.is_some()
            || self.state_storage.roar_state.is_some()
            || self.state_storage.pillar_state.is_some()
            || self.state_storage.knight_cut_state.is_some()
            || self.state_storage.tian_shen_xia_fan_state.is_some()
            || self.state_storage.wangsheng_state.is_some()
            || self.state_storage.blood_loss_state.is_some()
            || self.state_storage.kerosene_state.is_some()
            || self.state_storage.leaf_cut_state.is_some()
            || self.state_storage.leaf_cut_2_state.is_some()
            || self.state_storage.leaf_cut_3_state.is_some()
            || !self.state_storage.strike_states.is_empty()
            || !self.state_storage.battle_fairy_attribute_states.is_empty()
            || !self.state_storage.particular_states.is_empty()
            || !self.state_storage.consumable_restore_states.is_empty()
            || !self.state_storage.team_recruitment_states.is_empty()
            || !self.state_storage.defense_shields.is_empty()
            || !self.state_storage.change_body_states.is_empty()
            || !self.state_storage.extended_states.is_empty()
            || !self.state_storage.undead_states.is_empty()
            || self.state_storage.ride_state.is_some()
            || !self.state_storage.script_states.is_empty()
    }

    pub(crate) fn restore_automatic_hp_mp_states(
        &mut self,
        properties: super::player::PlayerCombatProperties,
    ) {
        while let Some(offset) = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .find(|offset| {
                read_u32(&self.ex_states, *offset).is_some_and(is_automatic_restore_state_id)
            })
        {
            let _ = self.remove_serialized_state_record_at(offset, AUTOMATIC_RESTORE_STATE_BYTES);
        }
        let states = AutomaticRestoreState::restored(properties);
        for state in states {
            self.append_serialized_state_record(&state.encoded_for_install());
        }
        self.automatic_restore_states = states.into();
    }

    pub(crate) fn activate_loaded_automatic_restore_states(&mut self, now_ms: u32) -> usize {
        for state in &mut self.automatic_restore_states {
            *state = state.activate_loaded(now_ms);
        }
        self.automatic_restore_states.len()
    }

    pub(crate) fn begin_consumable_health_restore(
        &mut self,
        amount: u32,
        time_to_keep_ms: u32,
        frequency_ms: u32,
        interval_ms: u32,
        now_ms: impl FnMut() -> u32,
    ) -> bool {
        let Some(record) = self.consumable_restore_states.begin_health(
            amount,
            time_to_keep_ms,
            frequency_ms,
            interval_ms,
            now_ms,
        ) else {
            return false;
        };
        self.append_serialized_state_record(&record);
        true
    }

    pub(crate) fn begin_consumable_mana_restore(
        &mut self,
        amount: u32,
        time_to_keep_ms: u32,
        frequency_ms: u32,
        interval_ms: u32,
        now_ms: impl FnMut() -> u32,
    ) -> bool {
        let Some(record) = self.consumable_restore_states.begin_mana(
            amount,
            time_to_keep_ms,
            frequency_ms,
            interval_ms,
            now_ms,
        ) else {
            return false;
        };
        self.append_serialized_state_record(&record);
        true
    }

    pub(crate) const fn consumable_restore_state_count(&self) -> usize {
        self.state_storage.consumable_restore_states.len()
    }

    pub(crate) fn activate_loaded_consumable_restore_states(&mut self, now_ms: u32) -> usize {
        self.consumable_restore_states.activate_loaded(now_ms)
    }

    pub(crate) fn consumable_restore_state_is_health(&self, index: usize) -> Option<bool> {
        self.consumable_restore_states.is_health(index)
    }

    pub(crate) fn tick_consumable_restore_state(
        &mut self,
        index: usize,
        checked_at_ms: u32,
        current: u32,
        maximum: u32,
    ) -> Option<ConsumableRestoreMutation> {
        self.consumable_restore_states
            .tick(index, checked_at_ms, current, maximum)
    }

    pub(crate) fn consumable_restore_state_expired(
        &self,
        index: usize,
        checked_at_ms: u32,
    ) -> Option<bool> {
        self.consumable_restore_states.expired(index, checked_at_ms)
    }

    pub(crate) fn remove_consumable_restore_state(&mut self, index: usize) -> bool {
        let Some(state_id) = self.consumable_restore_states.state_id(index) else {
            return false;
        };
        let occurrence = (0..index)
            .filter(|known| self.consumable_restore_states.state_id(*known) == Some(state_id))
            .count();
        let serialized_offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .filter(|offset| read_i32(&self.ex_states, *offset) == Some(state_id))
            .nth(occurrence);
        if !self.consumable_restore_states.remove(index) {
            return false;
        }
        if let Some(offset) = serialized_offset {
            let amount = if state_id == RESTORE_HP_STATE_ID {
                RESTORE_HP_STATE_BYTES
            } else {
                RESTORE_MP_STATE_BYTES
            };
            let _ = self.remove_serialized_state_record_at(offset, amount);
        }
        true
    }

    pub(crate) fn particular_states(&self) -> &[ParticularState] {
        &self.particular_states
    }

    pub(crate) fn add_particular_state(
        &mut self,
        state: ParticularState,
    ) -> Option<ParticularState> {
        if self
            .particular_states
            .iter()
            .any(|stored| stored.additional_data() == state.additional_data())
        {
            return None;
        }
        self.append_serialized_state_record(&state.encoded());
        self.particular_states.push(state);
        Some(state)
    }

    pub(crate) fn take_particular_states(&mut self) -> Vec<ParticularState> {
        let states = std::mem::take(&mut self.particular_states);
        while self.remove_serialized_state_record(PARTICULAR_STATE_ID, PARTICULAR_STATE_BYTES) {}
        states
    }

    pub(crate) fn remove_particular_state_at(
        &mut self,
        index: usize,
    ) -> Option<ParticularState> {
        let state = (index < self.particular_states.len())
            .then(|| self.particular_states.remove(index))?;
        let offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .filter(|offset| read_u32(&self.ex_states, *offset) == Some(PARTICULAR_STATE_ID))
            .nth(index);
        if let Some(offset) = offset {
            let _ = self.remove_serialized_state_record_at(offset, PARTICULAR_STATE_BYTES);
        }
        Some(state)
    }

    pub(crate) fn team_recruitment_states(&self) -> &[CTeamState] {
        &self.team_recruitment_states
    }

    pub(crate) fn team_recruitment_state_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut CTeamState> {
        self.team_recruitment_states.get_mut(index)
    }

    pub(crate) fn attach_team_recruitment_state(&mut self, state: CTeamState) {
        self.append_serialized_state_record(&state.encoded_for_install());
        self.team_recruitment_states.push(state);
    }

    pub(crate) fn remove_team_recruitment_state_at(
        &mut self,
        index: usize,
    ) -> Option<CTeamState> {
        let state = (index < self.team_recruitment_states.len())
            .then(|| self.team_recruitment_states.remove(index))?;
        let offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .filter(|offset| read_i32(&self.ex_states, *offset) == Some(TEAM_STATE_ID))
            .nth(index);
        if let Some(offset) = offset
            && let Some(amount) = CTeamState::serialized_size(&self.ex_states, offset)
        {
            let _ = self.remove_serialized_state_record_at(offset, amount);
        }
        Some(state)
    }

    pub(crate) fn automatic_restore_state(&self, index: usize) -> Option<AutomaticRestoreState> {
        self.automatic_restore_states.get(index).copied()
    }

    pub(crate) fn automatic_restore_state_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut AutomaticRestoreState> {
        self.automatic_restore_states.get_mut(index)
    }

    pub(crate) const fn automatic_restore_state_count(&self) -> usize {
        self.state_storage.automatic_restore_states.len()
    }

    /// Точный фабричный диапазон `CMoveShape::AddState`: остальные ID не
    /// создают состояние. Значения принимают исходное знаковое представление
    /// сценария и сохраняются как поля `DWORD` конкретных классов.
    pub(crate) fn add_script_state(
        &mut self,
        state_id: i32,
        value1: i32,
        value2: i32,
        sufferer_is_gm: bool,
        started_at_ms: u32,
    ) -> Option<ScriptMoveState> {
        let state = ScriptMoveState::from_factory(
            state_id,
            value1,
            value2,
            sufferer_is_gm,
            started_at_ms,
        )?;
        self.append_serialized_state_record(&state.encoded_for_install());
        self.script_states.push(state);
        Some(state)
    }

    /// Точный `GetStateNumByStateID`: считает все живые экземпляры с данным
    /// базовым `CState::m_lID`, независимо от concrete owner-а состояния.
    pub(crate) fn state_count_by_state_id(&self, state_id: i32) -> u32 {
        let scripted = self
            .script_states
            .iter()
            .filter(|state| state.state_id() == state_id)
            .count();
        let consumable_restore = self.consumable_restore_states.count(state_id);
        let change_body = (state_id == 0x37)
            .then_some(self.change_body_states.len())
            .unwrap_or(0);
        let extended = self
            .extended_states
            .iter()
            .filter(|state| state.kind.state_id() as i32 == state_id)
            .count();
        let undead = (state_id == UNDEAD_STATE_ID as i32)
            .then_some(self.undead_states.len())
            .unwrap_or(0);
        let ride = usize::from(state_id == RIDE_STATE_ID as i32 && self.ride_state.is_some());
        let automatic_restore = self
            .automatic_restore_states
            .iter()
            .filter(|state| state.state_id() as i32 == state_id)
            .count();
        let particular = (state_id == PARTICULAR_STATE_ID as i32)
            .then_some(self.particular_states.len())
            .unwrap_or(0);
        let team_recruitment = (state_id == TEAM_STATE_ID)
            .then_some(self.team_recruitment_states.len())
            .unwrap_or(0);
        let callosity = usize::from(
            self.callosity_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let agility = usize::from(
            self.persistent_agility_family_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        ) + usize::from(
            self.agility_state_2
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let taiji = usize::from(
            self.taiji_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let enlarge_full_miss = usize::from(
            self.enlarge_full_miss_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let enlarge_max_hp = usize::from(
            self.enlarge_max_hp_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let enlarge_max_mp = usize::from(
            self.enlarge_max_mp_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let origin = usize::from(
            self.origin_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let hearten = usize::from(
            self.hearten_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let heal = self
            .heal_states
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        let fury = self
            .fury_states
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        let rage_break = usize::from(
            self.rage_break_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let boss_blue_fury = usize::from(
            self.boss_blue_fury_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let boss_blue_quake = usize::from(
            self.boss_blue_quake_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let cure = self.cure_states.iter()
            .filter(|state| state.skill_id() as i32 == state_id).count();
        let daub_poison = usize::from(
            self.daub_poison_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let seal = usize::from(
            self.seal_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let poison_arrow = usize::from(
            self.poison_arrow_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let poison_fog = usize::from(self.poison_fog_state.is_some_and(|state| state.skill_id() as i32 == state_id));
        let meteor_arrow = usize::from(self.meteor_arrow_state.is_some_and(|state| state.skill_id() as i32 == state_id));
        let spider_poison = usize::from(
            self.spider_poison_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let sprite_burn = usize::from(
            self.sprite_burn_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let spider_web = usize::from(
            self.spider_web_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let weak = usize::from(
            self.weak_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let god_bless = usize::from(self.god_bless_state.is_some_and(|state| state.skill_id() as i32 == state_id));
        let soul_collect = usize::from(self.soul_collect_state.is_some_and(|state| state.skill_id() as i32 == state_id));
        let knock_out = usize::from(
            self.knock_out_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let blind = usize::from(
            self.blind_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let boa_lock = usize::from(self.boa_lock_state.is_some_and(|state| state.skill_id() as i32 == state_id));
        let rush = usize::from(
            self.rush_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let rush_2 = usize::from(
            self.rush_2_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let roar = usize::from(
            self.roar_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let energy_holding = usize::from(
            self.energy_holding_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let pillar = usize::from(
            self.pillar_state.is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let knight_cut = usize::from(
            self.knight_cut_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let blood_loss = usize::from(
            self.blood_loss_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let leaf_cut = usize::from(
            self.leaf_cut_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let leaf_cut_2 = usize::from(
            self.leaf_cut_2_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let leaf_cut_3 = usize::from(
            self.leaf_cut_3_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let kerosene = usize::from(
            self.kerosene_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let strike = self
            .strike_states
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        let swordship = self
            .swordship_states
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        let wuxing = self
            .wuxing_states
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        let battle_fairy_attributes = self
            .battle_fairy_attribute_states
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        let tian_shen_xia_fan = usize::from(
            self.tian_shen_xia_fan_state
                .is_some_and(|state| state.state_id() as i32 == state_id),
        );
        let wangsheng = usize::from(
            self.wangsheng_state
                .is_some_and(|state| state.state_id() as i32 == state_id),
        );
        let shields = self
            .defense_shields
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        scripted
            .saturating_add(consumable_restore)
            .saturating_add(agility)
            .saturating_add(callosity)
            .saturating_add(taiji)
            .saturating_add(enlarge_full_miss)
            .saturating_add(enlarge_max_hp)
            .saturating_add(enlarge_max_mp)
            .saturating_add(origin)
            .saturating_add(hearten)
            .saturating_add(heal)
            .saturating_add(fury)
            .saturating_add(rage_break)
            .saturating_add(boss_blue_fury)
            .saturating_add(boss_blue_quake)
            .saturating_add(cure)
            .saturating_add(daub_poison)
            .saturating_add(seal)
            .saturating_add(poison_arrow)
            .saturating_add(poison_fog)
            .saturating_add(meteor_arrow)
            .saturating_add(spider_poison)
            .saturating_add(sprite_burn)
            .saturating_add(spider_web)
            .saturating_add(weak)
            .saturating_add(god_bless)
            .saturating_add(soul_collect)
            .saturating_add(knock_out)
            .saturating_add(blind)
            .saturating_add(boa_lock)
            .saturating_add(rush)
            .saturating_add(rush_2)
            .saturating_add(roar)
            .saturating_add(energy_holding)
            .saturating_add(pillar)
            .saturating_add(knight_cut)
            .saturating_add(blood_loss)
            .saturating_add(leaf_cut)
            .saturating_add(leaf_cut_2)
            .saturating_add(leaf_cut_3)
            .saturating_add(kerosene)
            .saturating_add(strike)
            .saturating_add(swordship)
            .saturating_add(wuxing)
            .saturating_add(battle_fairy_attributes)
            .saturating_add(tian_shen_xia_fan)
            .saturating_add(wangsheng)
            .saturating_add(shields)
            .saturating_add(automatic_restore)
            .saturating_add(particular)
            .saturating_add(team_recruitment)
            .saturating_add(change_body)
            .saturating_add(extended)
            .saturating_add(undead)
            .saturating_add(ride)
            .min(u32::MAX as usize) as u32
    }

    /// `GetStateBySkillID` просматривает канонические типизированные состояния
    /// по фактическому идентификатору навыка, а не по классу сетевой записи.
    pub(crate) fn has_state_by_skill_id(&self, state_id: u32) -> bool {
        self.persistent_agility_family_state
            .is_some_and(|state| state.skill_id() == state_id)
            || self
                .agility_state_2
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .callosity_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .taiji_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .enlarge_full_miss_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .enlarge_max_hp_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .enlarge_max_mp_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .origin_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .hearten_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .heal_states
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .fury_states
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .rage_break_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .boss_blue_fury_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .boss_blue_quake_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .cure_states.iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .daub_poison_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .seal_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .poison_arrow_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self.poison_fog_state.is_some_and(|state| state.skill_id() == state_id)
            || self.meteor_arrow_state.is_some_and(|state| state.skill_id() == state_id)
            || self
                .spider_poison_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .sprite_burn_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .spider_web_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .weak_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self.god_bless_state.is_some_and(|state| state.skill_id() == state_id)
            || self.soul_collect_state.is_some_and(|state| state.skill_id() == state_id)
            || self
                .knock_out_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self.blind_state.is_some_and(|state| state.skill_id() == state_id)
            || self.boa_lock_state.is_some_and(|state| state.skill_id() == state_id)
            || self.rush_state.is_some_and(|state| state.skill_id() == state_id)
            || self.rush_2_state.is_some_and(|state| state.skill_id() == state_id)
            || self.roar_state.is_some_and(|state| state.skill_id() == state_id)
            || self.energy_holding_state.is_some_and(|state| state.skill_id() == state_id)
            || self.pillar_state.is_some_and(|state| state.skill_id() == state_id)
            || self
                .knight_cut_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .blood_loss_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .leaf_cut_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .leaf_cut_2_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .leaf_cut_3_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .kerosene_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .swordship_states
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .strike_states
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .wuxing_states
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .battle_fairy_attribute_states
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .tian_shen_xia_fan_state
                .is_some_and(|state| state.state_id() == state_id)
            || self
                .wangsheng_state
                .is_some_and(|state| state.state_id() == state_id)
            || self
                .defense_shields
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .script_states
                .iter()
                .any(|state| state.state_id() as u32 == state_id)
            || self
                .automatic_restore_states
                .iter()
                .any(|state| state.state_id() == state_id)
            || (state_id == PARTICULAR_STATE_ID && !self.particular_states.is_empty())
            || (state_id == TEAM_STATE_ID as u32 && !self.team_recruitment_states.is_empty())
            || self.consumable_restore_states.contains(state_id)
            || (state_id == CHANGE_BODY_STATE_ID && !self.change_body_states.is_empty())
            || self
                .extended_states
                .iter()
                .any(|state| state.kind.state_id() == state_id)
            || self
                .undead_states
                .iter()
                .any(|state| state.state_id() == state_id)
            || (state_id == RIDE_STATE_ID && self.ride_state.is_some())
    }

    pub(crate) fn callosity_state(&self) -> Option<CallosityFamilyState> {
        self.state_storage.callosity_state
    }

    pub(crate) fn take_callosity_state(&mut self, skill_id: u32) -> Option<CallosityFamilyState> {
        let state = self.callosity_state.filter(|state| state.skill_id() == skill_id)?;
        self.callosity_state = None;
        self.remove_serialized_state_record(skill_id, CALLOSITY_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn begin_callosity_state(&mut self, state: CallosityFamilyState) {
        debug_assert!(self.callosity_state.is_none());
        self.append_serialized_state_record(&state.encoded_for_install());
        self.callosity_state = Some(state);
    }

    pub(crate) fn activate_loaded_callosity_state(&mut self, now_ms: u32) {
        self.callosity_state = self.callosity_state.map(|state| state.activate_loaded(now_ms));
    }

    pub(crate) fn swordship_states(&self) -> &[SwordshipState] {
        &self.swordship_states
    }

    /// Заменяет состояние в прежней позиции семейного списка, а новый ID
    /// добавляет в конец. Так сохраняется относительный порядок этих прибавок.
    pub(crate) fn replace_swordship_state(
        &mut self,
        state: SwordshipState,
    ) -> Option<SwordshipState> {
        self.remove_serialized_state_record(state.skill_id(), SWORDSHIP_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        if let Some(slot) = self
            .swordship_states
            .iter_mut()
            .find(|current| current.skill_id() == state.skill_id())
        {
            return Some(std::mem::replace(slot, state));
        }
        self.swordship_states.push(state);
        None
    }

    /// Замена сохраняет прежнюю позицию среди пяти стихийных состояний;
    /// новый skill ID добавляется в хвост, как в исходном `m_vStates`.
    pub(crate) fn replace_wuxing_state(
        &mut self,
        state: WuXingState,
    ) -> Option<WuXingState> {
        let serialized_offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .find(|offset| read_u32(&self.ex_states, *offset) == Some(state.skill_id()));
        if let Some(offset) = serialized_offset {
            let end = offset.saturating_add(WUXING_STATE_BYTES);
            if let Some(destination) = self.ex_states.get_mut(offset..end) {
                destination.copy_from_slice(&state.encoded());
            }
        } else {
            if self.ex_states.len() < 4 {
                self.ex_states.clear();
                LegacyWriter::new(&mut self.ex_states).write_u32(0);
            }
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
            self.ex_states.extend_from_slice(&state.encoded());
        }
        if let Some(slot) = self
            .wuxing_states
            .iter_mut()
            .find(|current| current.skill_id() == state.skill_id())
        {
            return Some(std::mem::replace(slot, state));
        }
        self.wuxing_states.push(state);
        None
    }

    pub(crate) fn wuxing_states(&self) -> &[WuXingState] {
        &self.wuxing_states
    }

    pub(crate) const fn taiji_state(&self) -> Option<TaiJiState> {
        self.state_storage.taiji_state
    }

    pub(crate) fn replace_taiji_state(&mut self, state: TaiJiState) -> Option<TaiJiState> {
        self.remove_serialized_state_record(state.skill_id(), TAIJI_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        self.taiji_state.replace(state)
    }

    pub(crate) fn replace_enlarge_max_hp_state(
        &mut self,
        state: EnlargeMaxHpState,
    ) -> Option<EnlargeMaxHpState> {
        self.remove_serialized_state_record(state.skill_id(), ENLARGE_MAX_HP_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        self.enlarge_max_hp_state.replace(state)
    }

    pub(crate) fn replace_enlarge_full_miss_state(
        &mut self,
        state: EnlargeFullMissState,
    ) -> Option<EnlargeFullMissState> {
        let state_id = state.skill_id();
        let offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .find(|offset| read_u32(&self.ex_states, *offset) == Some(state_id));
        if let Some(offset) = offset {
            if let Some(destination) = self.ex_states.get_mut(offset..offset + ENLARGE_FULL_MISS_STATE_BYTES) {
                destination.copy_from_slice(&state.encoded());
            }
        } else {
            if self.ex_states.len() < 4 {
                self.ex_states.clear();
                LegacyWriter::new(&mut self.ex_states).write_u32(0);
            }
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
            self.ex_states.extend_from_slice(&state.encoded());
        }
        self.enlarge_full_miss_state.replace(state)
    }

    pub(crate) const fn enlarge_full_miss_state(&self) -> Option<EnlargeFullMissState> {
        self.state_storage.enlarge_full_miss_state
    }

    pub(crate) fn replace_enlarge_max_mp_state(
        &mut self,
        state: EnlargeMaxMpState,
    ) -> Option<EnlargeMaxMpState> {
        self.remove_serialized_state_record(state.skill_id(), ENLARGE_MAX_MP_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        self.enlarge_max_mp_state.replace(state)
    }

    pub(crate) const fn enlarge_max_hp_state(&self) -> Option<EnlargeMaxHpState> {
        self.state_storage.enlarge_max_hp_state
    }

    pub(crate) const fn enlarge_max_mp_state(&self) -> Option<EnlargeMaxMpState> {
        self.state_storage.enlarge_max_mp_state
    }

    pub(crate) fn replace_origin_state(&mut self, state: OriginState) -> Option<OriginState> {
        self.remove_serialized_state_record(state.skill_id(), ORIGIN_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        self.origin_state.replace(state)
    }

    pub(crate) const fn origin_state(&self) -> Option<OriginState> {
        self.state_storage.origin_state
    }

    pub(crate) fn replace_hearten_state(&mut self, state: HeartenState) -> Option<HeartenState> {
        let previous = self.hearten_state.take();
        self.remove_serialized_state_record(state.skill_id(), HEARTEN_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.hearten_state = Some(state);
        previous
    }

    pub(crate) const fn hearten_state(&self) -> Option<HeartenState> {
        self.state_storage.hearten_state
    }

    pub(crate) fn take_expired_hearten_state(&mut self, now_ms: u32) -> Option<HeartenState> {
        self.hearten_state.filter(|state| state.expired(now_ms))?;
        let state = self.hearten_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), HEARTEN_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn activate_loaded_hearten_state(&mut self, now_ms: u32) {
        if let Some(state) = &mut self.hearten_state {
            state.activate_loaded(now_ms);
        }
    }

    pub(crate) fn replace_heal_state(
        &mut self,
        removed_skill_id: u32,
        state: HealState,
    ) -> Option<HealState> {
        self.remove_serialized_state_record(removed_skill_id, HEAL_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        let previous = self
            .heal_states
            .iter()
            .position(|candidate| candidate.skill_id() == removed_skill_id)
            .map(|position| self.heal_states.remove(position));
        self.heal_states.push(state);
        previous
    }

    pub(crate) fn remove_serialized_heal_states(&mut self, skill_ids: &[u32]) {
        for skill_id in skill_ids {
            self.remove_serialized_state_record(*skill_id, HEAL_STATE_BYTES);
        }
    }

    pub(crate) fn remove_serialized_heal_state(&mut self, skill_id: u32, occurrence: usize) {
        if let Some(offset) = known_state_record_offsets(&self.ex_states).into_iter()
            .filter(|offset| read_u32(&self.ex_states, *offset) == Some(skill_id))
            .nth(occurrence)
        {
            self.remove_serialized_state_record_at(offset, HEAL_STATE_BYTES);
        }
    }

    pub(crate) fn activate_loaded_heal_states(&mut self, now_ms: u32) -> Vec<HealState> {
        for state in &mut self.heal_states {
            state.activate_loaded(now_ms);
        }
        self.heal_states.clone()
    }

    pub(crate) fn take_heal_states(&mut self) -> Vec<HealState> {
        std::mem::take(&mut self.heal_states)
    }

    pub(crate) fn restore_heal_states(&mut self, states: Vec<HealState>) {
        debug_assert!(self.heal_states.is_empty());
        self.heal_states = states;
    }

    pub(crate) fn push_fury_state(&mut self, state: FuryState) {
        self.append_serialized_state_record(&state.encoded_for_install());
        self.fury_states.push(state);
    }

    pub(crate) fn activate_loaded_fury_states(&mut self, now_ms: u32) -> Vec<FuryState> {
        for state in &mut self.fury_states {
            *state = state.activate_loaded(now_ms);
        }
        self.fury_states.clone()
    }

    pub(crate) fn fury_states(&self) -> &[FuryState] {
        &self.fury_states
    }

    pub(crate) fn remove_fury_state(&mut self, position: usize) -> Option<FuryState> {
        self.fury_states.get(position)?;
        let serialized_offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .filter(|offset| read_u32(&self.ex_states, *offset) == Some(FURY_STATE_SKILL_ID))
            .nth(position);
        let state = self.fury_states.remove(position);
        if let Some(offset) = serialized_offset {
            self.remove_serialized_state_record_at(offset, FURY_STATE_BYTES);
        }
        Some(state)
    }

    pub(crate) const fn rage_break_state(&self) -> Option<RageBreakState> {
        self.state_storage.rage_break_state
    }

    pub(crate) fn replace_rage_break_state(&mut self, state: RageBreakState) -> Option<RageBreakState> {
        self.remove_serialized_state_record(state.skill_id(), RAGE_BREAK_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.rage_break_state.replace(state)
    }

    pub(crate) fn activate_loaded_rage_break_state(&mut self, now_ms: u32) -> Option<RageBreakState> {
        let state = self.rage_break_state?.activate_loaded(now_ms);
        self.rage_break_state = Some(state);
        Some(state)
    }

    pub(crate) fn take_rage_break_state(&mut self) -> Option<RageBreakState> {
        let state = self.rage_break_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), RAGE_BREAK_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn restart_rage_break_state(&mut self, now_ms: u32) -> bool {
        let Some(state) = self.rage_break_state.as_mut() else {
            return false;
        };
        *state = state.activate_loaded(now_ms);
        true
    }

    pub(crate) fn take_expired_rage_break_state(&mut self, now_ms: u32) -> Option<RageBreakState> {
        self.rage_break_state.filter(|state| state.expired(now_ms))?;
        let state = self.rage_break_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), RAGE_BREAK_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_boss_blue_fury_state(&mut self) -> Option<BossBlueFuryState> {
        let state = self.boss_blue_fury_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), BOSS_BLUE_FURY_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn begin_boss_blue_fury_state(&mut self, state: BossBlueFuryState) {
        debug_assert!(self.boss_blue_fury_state.is_none());
        self.append_serialized_state_record(&state.encoded_for_install());
        self.boss_blue_fury_state = Some(state);
    }

    pub(crate) fn boss_blue_fury_state(&self) -> Option<BossBlueFuryState> {
        self.boss_blue_fury_state
    }

    pub(crate) fn tick_boss_blue_fury_state(
        &mut self,
        now_ms: u32,
    ) -> Option<(BossBlueFuryState, BossBlueFuryTick)> {
        let state = self.boss_blue_fury_state.as_mut()?;
        let tick = state.tick(now_ms);
        let snapshot = *state;
        if tick.expired {
            self.boss_blue_fury_state = None;
            self.remove_serialized_state_record(snapshot.skill_id(), BOSS_BLUE_FURY_STATE_BYTES);
        }
        Some((snapshot, tick))
    }

    pub(crate) fn activate_loaded_boss_blue_fury_state(
        &mut self,
        now_ms: u32,
    ) -> Option<BossBlueFuryState> {
        let mut state = self.boss_blue_fury_state?;
        state.activate_loaded(now_ms);
        self.boss_blue_fury_state = Some(state);
        Some(state)
    }

    pub(crate) fn replace_boss_blue_quake_state(
        &mut self,
        state: BossBlueQuakeState,
    ) -> Option<BossBlueQuakeState> {
        self.remove_serialized_state_record(state.skill_id(), BOSS_BLUE_QUAKE_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.curable_state_order.insert(state.skill_id());
        self.boss_blue_quake_state.replace(state)
    }

    pub(crate) fn activate_loaded_boss_blue_quake_state(
        &mut self,
        now_ms: u32,
    ) -> Option<BossBlueQuakeState> {
        let state = self.boss_blue_quake_state?.activate_loaded(now_ms);
        self.boss_blue_quake_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) fn take_expired_boss_blue_quake_state(
        &mut self,
        now_ms: u32,
    ) -> Option<BossBlueQuakeState> {
        self.boss_blue_quake_state.filter(|state| state.expired(now_ms))?;
        let state = self.boss_blue_quake_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), BOSS_BLUE_QUAKE_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_boss_blue_quake_state(&mut self) -> Option<BossBlueQuakeState> {
        let state = self.boss_blue_quake_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), BOSS_BLUE_QUAKE_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_mana_shield_state(
        &mut self,
        state: ManaShieldState,
    ) -> Option<ManaShieldState> {
        let previous = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
            .map(|position| self.defense_shields.remove(position))
            .and_then(|candidate| match candidate {
                DefenseShieldState::Life(_) => None,
                DefenseShieldState::Mana(previous) => Some(previous),
                DefenseShieldState::Machine(_) => None,
                DefenseShieldState::Promotion(_) => None,
            });
        self.remove_serialized_state_record(state.skill_id(), MANA_SHIELD_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.defense_shields.push(DefenseShieldState::Mana(state));
        previous
    }

    pub(crate) fn replace_machine_shield_state(
        &mut self,
        state: MachineShieldState,
    ) -> Option<MachineShieldState> {
        let previous = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
            .map(|position| self.defense_shields.remove(position))
            .and_then(|candidate| match candidate {
                DefenseShieldState::Life(_) => None,
                DefenseShieldState::Machine(previous) => Some(previous),
                DefenseShieldState::Mana(_) => None,
                DefenseShieldState::Promotion(_) => None,
            });
        self.remove_serialized_state_record(state.skill_id(), MACHINE_SHIELD_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.defense_shields
            .push(DefenseShieldState::Machine(state));
        previous
    }

    pub(crate) fn replace_life_shield_state(
        &mut self,
        state: LifeShieldState,
    ) -> Option<LifeShieldState> {
        let previous = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
            .map(|position| self.defense_shields.remove(position))
            .and_then(|candidate| match candidate {
                DefenseShieldState::Life(previous) => Some(previous),
                DefenseShieldState::Machine(_)
                | DefenseShieldState::Mana(_)
                | DefenseShieldState::Promotion(_) => None,
            });
        self.remove_serialized_state_record(state.skill_id(), LIFE_SHIELD_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.defense_shields.push(DefenseShieldState::Life(state));
        previous
    }

    /// Повторное наложение вызывает Restart прежнего состояния без замены
    /// его параметров и без нового Begin.
    pub(crate) fn begin_promotion_state(&mut self, state: PromotionState) -> bool {
        if let Some(position) = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
        {
            if let DefenseShieldState::Promotion(previous) = &mut self.defense_shields[position] {
                previous.restart(state.started_at_ms());
            }
            return false;
        }
        self.append_serialized_state_record(&state.encoded_for_install());
        self.defense_shields
            .push(DefenseShieldState::Promotion(state));
        true
    }

    pub(crate) fn promotion_magic_attack_factor(&self) -> Option<u16> {
        self.defense_shields.iter().find_map(|state| match state {
            DefenseShieldState::Promotion(state) => Some(state.magic_attack_factor()),
            _ => None,
        })
    }

    pub(crate) fn promotion_heal_recover_factor(&self) -> Option<u16> {
        self.defense_shields.iter().find_map(|state| match state {
            DefenseShieldState::Promotion(state) => Some(state.heal_recover_factor()),
            _ => None,
        })
    }

    pub(crate) fn take_expired_promotion_state(
        &mut self,
        now_ms: u32,
    ) -> Option<PromotionState> {
        let position = self.defense_shields.iter().position(|state| {
            matches!(state, DefenseShieldState::Promotion(promotion) if promotion.expired(now_ms))
        })?;
        match self.defense_shields.remove(position) {
            DefenseShieldState::Promotion(state) => {
                self.remove_serialized_state_record(state.skill_id(), PROMOTION_STATE_BYTES);
                Some(state)
            }
            _ => unreachable!("позиция состояния Promotion проверена"),
        }
    }

    pub(crate) fn defense_shields(&self) -> &[DefenseShieldState] {
        &self.defense_shields
    }

    pub(crate) fn remove_defense_shield(&mut self, skill_id: u32) -> Option<DefenseShieldState> {
        let position = self.defense_shields.iter().position(|state| state.skill_id() == skill_id)?;
        let state = self.defense_shields.remove(position);
        let bytes = match state {
            DefenseShieldState::Mana(_) => MANA_SHIELD_STATE_BYTES,
            DefenseShieldState::Machine(_) => MACHINE_SHIELD_STATE_BYTES,
            DefenseShieldState::Life(_) => LIFE_SHIELD_STATE_BYTES,
            DefenseShieldState::Promotion(_) => PROMOTION_STATE_BYTES,
        };
        self.remove_serialized_state_record(skill_id, bytes);
        Some(state)
    }

    pub(crate) fn activate_loaded_persisted_defense_shields(
        &mut self,
        now_ms: u32,
    ) -> Vec<DefenseShieldState> {
        let mut loaded = Vec::new();
        for state in &mut self.defense_shields {
            match state {
                DefenseShieldState::Mana(state) => {
                    state.activate_loaded(now_ms);
                    loaded.push(DefenseShieldState::Mana(*state));
                }
                DefenseShieldState::Machine(state) => {
                    state.activate_loaded(now_ms);
                    loaded.push(DefenseShieldState::Machine(*state));
                }
                DefenseShieldState::Life(state) => {
                    state.activate_loaded(now_ms);
                    loaded.push(DefenseShieldState::Life(*state));
                }
                DefenseShieldState::Promotion(state) => {
                    state.activate_loaded(now_ms);
                    loaded.push(DefenseShieldState::Promotion(*state));
                }
            }
        }
        loaded
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
        for state in &mut self.extended_states { state.shift_serialized_offset_after(offset, amount); }
        for state in &mut self.change_body_states { state.shift_serialized_offset_after(offset, amount); }
        for state in &mut self.undead_states { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.leaf_cut_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.leaf_cut_3_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.kerosene_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.poison_fog_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.ride_state { state.shift_serialized_offset_after(offset, amount); }
        true
    }

    pub(crate) fn take_defense_shields(&mut self) -> Vec<DefenseShieldState> {
        std::mem::take(&mut self.defense_shields)
    }

    pub(crate) fn restore_defense_shields(&mut self, states: Vec<DefenseShieldState>) {
        self.defense_shields = states;
    }

    pub(crate) fn push_cure_state(&mut self, state: CureState) {
        self.append_serialized_state_record(&state.encoded());
        self.cure_states.push(state);
    }

    pub(crate) fn cure_state_replacement_offset(&self) -> Option<usize> {
        known_state_record_offsets(&self.ex_states).into_iter()
            .find(|offset| read_u32(&self.ex_states, *offset) == Some(CURE_STATE_SKILL_ID))
    }

    pub(crate) fn insert_replacement_cure_state(&mut self, state: CureState, offset: usize) {
        let amount = CURE_STATE_BYTES;
        self.ex_states.splice(offset..offset, state.encoded());
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        for known in &mut self.extended_states { known.shift_serialized_offset_for_insert(offset, amount); }
        for known in &mut self.change_body_states { known.shift_serialized_offset_for_insert(offset, amount); }
        for known in &mut self.undead_states { known.shift_serialized_offset_for_insert(offset, amount); }
        if let Some(known) = &mut self.leaf_cut_state { known.shift_serialized_offset_for_insert(offset, amount); }
        if let Some(known) = &mut self.leaf_cut_3_state { known.shift_serialized_offset_for_insert(offset, amount); }
        if let Some(known) = &mut self.kerosene_state { known.shift_serialized_offset_for_insert(offset, amount); }
        if let Some(known) = &mut self.poison_fog_state { known.shift_serialized_offset_for_insert(offset, amount); }
        if let Some(known) = &mut self.meteor_arrow_state { known.shift_serialized_offset_for_insert(offset, amount); }
        if let Some(known) = &mut self.ride_state { known.shift_serialized_offset_for_insert(offset, amount); }
        self.cure_states.insert(0, state);
    }

    pub(crate) fn activate_loaded_cure_states(&mut self, now_ms: u32) -> Vec<CureState> {
        for state in &mut self.cure_states {
            state.activate_loaded(now_ms);
        }
        self.cure_states.clone()
    }

    pub(crate) fn cure_states(&self) -> &[CureState] {
        &self.state_storage.cure_states
    }

    pub(crate) fn cure_state(&self) -> Option<CureState> {
        self.cure_states().first().copied()
    }

    pub(crate) fn take_cure_state(&mut self) -> Option<CureState> {
        self.remove_cure_state(0)
    }

    pub(crate) fn remove_cure_state(&mut self, position: usize) -> Option<CureState> {
        let state = *self.cure_states.get(position)?;
        if let Some(offset) = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .filter(|offset| read_u32(&self.ex_states, *offset) == Some(CURE_STATE_SKILL_ID))
            .nth(position)
        {
            self.remove_serialized_state_record_at(offset, CURE_STATE_BYTES);
        }
        self.cure_states.remove(position);
        Some(state)
    }

    pub(crate) fn replace_daub_poison_state(
        &mut self,
        state: DaubPoisonState,
    ) -> Option<DaubPoisonState> {
        self.remove_serialized_state_record(state.skill_id(), DAUB_POISON_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.daub_poison_state.replace(state)
    }
    pub(crate) fn activate_loaded_daub_poison_state(&mut self, now_ms: u32) -> Option<DaubPoisonState> { let state = self.daub_poison_state?.activate_loaded(now_ms); self.daub_poison_state = Some(state); Some(state) }

    pub(crate) fn take_expired_daub_poison_state(
        &mut self,
        now_ms: u32,
    ) -> Option<DaubPoisonState> {
        self.daub_poison_state
            .filter(|state| state.expired(now_ms))?;
        let state = self.daub_poison_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), DAUB_POISON_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_seal_state(&mut self, state: SealState) -> Option<SealState> {
        self.remove_serialized_state_record(state.skill_id(), SEAL_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.blind_state_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.seal_state.replace(state)
    }

    pub(crate) fn activate_loaded_seal_state(&mut self, now_ms: u32) -> Option<SealState> {
        let state = self.seal_state?.activate_loaded(now_ms);
        self.seal_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) fn take_expired_seal_state(&mut self, now_ms: u32) -> Option<SealState> {
        let state = self.seal_state.filter(|state| state.expired(now_ms))?;
        self.seal_state = None;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), SEAL_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_seal_state(&mut self) -> Option<SealState> {
        let state = self.seal_state.take()?;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), SEAL_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_poison_arrow_state(
        &mut self,
        state: PoisonArrowState,
    ) -> Option<PoisonArrowState> {
        self.periodic_attack_order.insert(state.skill_id());
        let previous = self.poison_arrow_state.replace(state);
        let serialized_exists = known_state_record_offsets(&self.ex_states)
            .into_iter().any(|offset| read_u32(&self.ex_states, offset) == Some(state.skill_id()));
        if previous.is_some() && serialized_exists {
            update_known_state_record(&mut self.ex_states, state.skill_id(), &state.encoded_for_install());
        } else if !serialized_exists {
            self.append_serialized_state_record(&state.encoded_for_install());
        }
        previous
    }

    pub(crate) fn take_poison_arrow_state_for_ai(&mut self) -> Option<PoisonArrowState> {
        self.poison_arrow_state.take()
    }

    pub(crate) fn finish_poison_arrow_state(&mut self, state: PoisonArrowState) {
        self.periodic_attack_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), POISON_ARROW_STATE_BYTES);
    }

    pub(crate) fn activate_loaded_poison_arrow_state(&mut self, now_ms: u32) -> Option<PoisonArrowState> {
        let mut state = self.poison_arrow_state?;
        state.activate_loaded(now_ms);
        self.poison_arrow_state = Some(state);
        Some(state)
    }

    pub(crate) fn replace_spider_poison_state(
        &mut self,
        state: SpiderPoisonState,
    ) -> Option<SpiderPoisonState> {
        let exists = known_state_record_offsets(&self.ex_states).into_iter().any(|offset| read_u32(&self.ex_states, offset) == Some(state.skill_id()));
        if exists { update_known_state_record(&mut self.ex_states, state.skill_id(), &state.encoded_for_install()); } else { self.append_serialized_state_record(&state.encoded_for_install()); }
        self.periodic_attack_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.spider_poison_state.replace(state)
    }
    pub(crate) fn activate_loaded_spider_poison_state(&mut self, now_ms: u32) -> Option<SpiderPoisonState> { let mut state = self.spider_poison_state?; state.activate_loaded(now_ms); self.spider_poison_state = Some(state); Some(state) }

    pub(crate) fn take_spider_poison_state_for_ai(&mut self) -> Option<SpiderPoisonState> {
        self.spider_poison_state.take()
    }
    pub(crate) fn restore_spider_poison_state_after_ai(&mut self, state: SpiderPoisonState) { debug_assert!(self.spider_poison_state.is_none()); self.spider_poison_state = Some(state); }
    pub(crate) fn finish_spider_poison_state_after_ai(&mut self) { self.remove_serialized_state_record(super::skills::spiderpoison::SPIDER_POISON_SKILL_ID, SPIDER_POISON_STATE_BYTES); self.finish_periodic_attack_state(super::skills::spiderpoison::SPIDER_POISON_SKILL_ID); }

    pub(crate) fn take_spider_poison_state(&mut self) -> Option<SpiderPoisonState> {
        let state = self.spider_poison_state.take()?;
        self.periodic_attack_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), SPIDER_POISON_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_sprite_burn_state(
        &mut self,
        state: SpriteBurnState,
    ) -> Option<SpriteBurnState> {
        let exists = known_state_record_offsets(&self.ex_states).into_iter()
            .any(|offset| read_u32(&self.ex_states, offset) == Some(state.skill_id()));
        if exists { update_known_state_record(&mut self.ex_states, state.skill_id(), &state.encoded_for_install()); }
        else { self.append_serialized_state_record(&state.encoded_for_install()); }
        self.periodic_attack_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.sprite_burn_state.replace(state)
    }

    pub(crate) fn activate_loaded_sprite_burn_state(&mut self, now_ms: u32) -> Option<SpriteBurnState> {
        let mut state = self.sprite_burn_state?; state.activate_loaded(now_ms); self.sprite_burn_state = Some(state); Some(state)
    }

    pub(crate) fn take_sprite_burn_state_for_ai(&mut self) -> Option<SpriteBurnState> {
        self.sprite_burn_state.take()
    }

    pub(crate) fn restore_sprite_burn_state_after_ai(&mut self, state: SpriteBurnState) { debug_assert!(self.sprite_burn_state.is_none()); self.sprite_burn_state = Some(state); }

    pub(crate) fn finish_sprite_burn_state_after_ai(&mut self) {
        self.remove_serialized_state_record(super::skills::spriteburn::SPRITE_BURN_SKILL_ID, SPRITE_BURN_STATE_BYTES);
        self.finish_periodic_attack_state(super::skills::spriteburn::SPRITE_BURN_SKILL_ID);
    }

    pub(crate) fn take_sprite_burn_state(&mut self) -> Option<SpriteBurnState> {
        let state = self.sprite_burn_state.take()?;
        self.periodic_attack_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), SPRITE_BURN_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_spider_web_state(
        &mut self,
        state: SpiderWebState,
    ) -> Option<SpiderWebState> {
        self.remove_serialized_state_record(state.skill_id(), SPIDER_WEB_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.blind_state_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.spider_web_state.replace(state)
    }

    pub(crate) fn activate_loaded_spider_web_state(&mut self, now_ms: u32) -> Option<SpiderWebState> {
        let state = self.spider_web_state?.activate_loaded(now_ms);
        self.spider_web_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) const fn weak_state(&self) -> Option<WeakState> {
        self.state_storage.weak_state
    }

    pub(crate) fn replace_weak_state(&mut self, state: WeakState) -> Option<WeakState> {
        self.remove_serialized_state_record(state.skill_id(), WEAK_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        self.weak_state.replace(state)
    }

    pub(crate) fn activate_loaded_weak_state(&self) -> Option<WeakState> { self.weak_state }

    pub(crate) fn take_weak_state(&mut self) -> Option<WeakState> {
        let state = self.weak_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), WEAK_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_weak_state_outside(&mut self, tile_x: i32, tile_y: i32) -> Option<WeakState> {
        let state = self.weak_state.filter(|state| !state.contains(tile_x, tile_y))?;
        self.weak_state = None;
        self.remove_serialized_state_record(state.skill_id(), WEAK_STATE_BYTES);
        Some(state)
    }

    pub(crate) const fn god_bless_state(&self) -> Option<GodBlessState> { self.state_storage.god_bless_state }
    pub(crate) fn replace_god_bless_state(&mut self, state: GodBlessState) -> Option<GodBlessState> {
        let previous = self.god_bless_state.take();
        if let Some(previous) = previous { self.remove_serialized_state_record(previous.skill_id(), GOD_BLESS_STATE_BYTES); }
        self.append_serialized_state_record(&state.encoded_for_install());
        self.god_bless_state = Some(state);
        previous
    }
    pub(crate) fn activate_loaded_god_bless_state(&mut self, now_ms: u32) -> Option<GodBlessState> {
        let state = self.god_bless_state?.activate_loaded(now_ms);
        self.god_bless_state = Some(state);
        Some(state)
    }
    pub(crate) fn take_god_bless_state(&mut self, skill_id: u32) -> Option<GodBlessState> {
        let state = self
            .god_bless_state
            .filter(|state| state.skill_id() == skill_id)?;
        self.god_bless_state = None;
        self.remove_serialized_state_record(state.skill_id(), GOD_BLESS_STATE_BYTES);
        Some(state)
    }
    pub(crate) const fn roar_state(&self) -> Option<RoarState> { self.state_storage.roar_state }
    pub(crate) fn replace_roar_state(&mut self, state: RoarState) -> Option<RoarState> {
        self.remove_serialized_state_record(state.skill_id(), ROAR_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.roar_state.replace(state)
    }
    pub(crate) fn activate_loaded_roar_state(&mut self, now_ms: u32) -> Option<RoarState> {
        let state = self.roar_state?.activate_loaded(now_ms);
        self.roar_state = Some(state);
        Some(state)
    }
    pub(crate) fn take_expired_roar_state(&mut self, now_ms: u32) -> Option<RoarState> {
        let state = self.roar_state.filter(|state| state.expired(now_ms))?;
        self.roar_state = None;
        self.remove_serialized_state_record(state.skill_id(), ROAR_STATE_BYTES);
        Some(state)
    }
    pub(crate) const fn energy_holding_state(&self) -> Option<EnergyHoldingState> { self.state_storage.energy_holding_state }
    pub(crate) fn energy_holding_state_mut(&mut self) -> Option<&mut EnergyHoldingState> { self.state_storage.energy_holding_state.as_mut() }
    pub(crate) fn begin_energy_holding_state(&mut self, state: EnergyHoldingState) {
        self.remove_serialized_state_record(state.skill_id(), ENERGY_HOLDING_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        self.state_storage.energy_holding_state = Some(state);
    }
    pub(crate) fn take_energy_holding_state(&mut self) -> Option<EnergyHoldingState> {
        let state = self.state_storage.energy_holding_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), ENERGY_HOLDING_STATE_BYTES);
        Some(state)
    }
    /// Восстанавливает относительный порядок уже типизированных property-state
    /// по byte-exact записям исходного `m_vStates`. Повторные экземпляры одного
    /// ID выбираются последовательно, поэтому два одинаковых состояния не
    /// схлопываются в одно при пересчёте свойств.
    pub(crate) fn ordered_player_property_states(&self) -> Vec<PlayerPropertyState> {
        let mut ordered = Vec::new();
        let mut swordship_occurrences = BTreeMap::new();
        let mut wuxing_occurrences = BTreeMap::new();
        let mut battle_fairy_occurrences = BTreeMap::new();
        let mut script_occurrences = BTreeMap::new();
        let mut undead_occurrences = BTreeMap::new();
        let mut extended_occurrences = BTreeMap::new();
        let mut fury_occurrences = BTreeMap::new();

        for offset in known_state_record_offsets(&self.ex_states) {
            let Some(state_id) = read_u32(&self.ex_states, offset) else {
                continue;
            };
            let state = self
                .persistent_agility_family_state
                .filter(|state| state.skill_id() == state_id)
                .map(PlayerPropertyState::PersistentAgility)
                .or_else(|| {
                    self.agility_state_2
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::Agility2)
                })
                .or_else(|| {
                    self.taiji_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::TaiJi)
                })
                .or_else(|| {
                    self.enlarge_max_hp_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::EnlargeMaxHp)
                })
                .or_else(|| {
                    self.enlarge_max_mp_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::EnlargeMaxMp)
                })
                .or_else(|| {
                    self.enlarge_full_miss_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::EnlargeFullMiss)
                })
                .or_else(|| {
                    self.origin_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::Origin)
                })
                .or_else(|| {
                    self.hearten_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::Hearten)
                })
                .or_else(|| {
                    self.callosity_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::Callosity)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.swordship_states,
                        state_id,
                        &mut swordship_occurrences,
                        SwordshipState::skill_id,
                    )
                    .map(PlayerPropertyState::Swordship)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.wuxing_states,
                        state_id,
                        &mut wuxing_occurrences,
                        WuXingState::skill_id,
                    )
                    .map(PlayerPropertyState::WuXing)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.battle_fairy_attribute_states,
                        state_id,
                        &mut battle_fairy_occurrences,
                        BattleFairyAttributeState::skill_id,
                    )
                    .map(PlayerPropertyState::BattleFairyAttribute)
                })
                .or_else(|| {
                    self.tian_shen_xia_fan_state
                        .filter(|state| state.state_id() == state_id)
                        .map(PlayerPropertyState::TianShenXiaFan)
                })
                .or_else(|| {
                    self.weak_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::Weak)
                })
                .or_else(|| {
                    self.poison_fog_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::PoisonFog)
                })
                .or_else(|| {
                    self.god_bless_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::GodBless)
                })
                .or_else(|| {
                    self.roar_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::Roar)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.script_states,
                        state_id,
                        &mut script_occurrences,
                        |state| state.state_id() as u32,
                    )
                    .map(PlayerPropertyState::Script)
                })
                .or_else(|| {
                    next_ordered_cloned_state(
                        &self.undead_states,
                        state_id,
                        &mut undead_occurrences,
                        |_| UNDEAD_STATE_ID,
                    )
                    .map(PlayerPropertyState::Undead)
                })
                .or_else(|| {
                    next_ordered_cloned_state(
                        &self.extended_states,
                        state_id,
                        &mut extended_occurrences,
                        |state| state.kind.state_id(),
                    )
                    .map(PlayerPropertyState::Extended)
                })
                .or_else(|| {
                    self.active_change_body_state()
                        .filter(|state| {
                            state.serialized_span().map(|(state_offset, _)| state_offset)
                                == Some(offset)
                        })
                        .cloned()
                        .map(PlayerPropertyState::ChangeBody)
                })
                .or_else(|| {
                    self.ride_state
                        .as_ref()
                        .filter(|state| {
                            state.serialized_span().map(|(state_offset, _)| state_offset)
                                == Some(offset)
                        })
                        .cloned()
                        .map(PlayerPropertyState::Ride)
                })
                .or_else(|| {
                    self.rage_break_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(PlayerPropertyState::RageBreak)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.fury_states,
                        state_id,
                        &mut fury_occurrences,
                        FuryState::skill_id,
                    )
                    .map(PlayerPropertyState::Fury)
                })
                .or_else(|| {
                    self.wangsheng_state
                        .filter(|state| state.state_id() == state_id)
                        .map(PlayerPropertyState::Wangsheng)
                });
            if let Some(state) = state {
                ordered.push(state);
            }
        }
        ordered
    }

    /// Проецирует monster-варианты `OnUpdateProperties` из того же общего
    /// `m_vStates`. Это сохраняет некоммутативный порядок процентных,
    /// аддитивных и заменяющих формул атаки и стихийного модификатора.
    pub(crate) fn ordered_monster_property_states(&self) -> Vec<MonsterPropertyState> {
        let mut ordered = Vec::new();
        let mut swordship_occurrences = BTreeMap::new();
        let mut battle_fairy_occurrences = BTreeMap::new();
        let mut fury_occurrences = BTreeMap::new();

        for offset in known_state_record_offsets(&self.ex_states) {
            let Some(state_id) = read_u32(&self.ex_states, offset) else {
                continue;
            };
            let state = self
                .taiji_state
                .filter(|state| state.skill_id() == state_id)
                .map(MonsterPropertyState::TaiJi)
                .or_else(|| {
                    self.origin_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(MonsterPropertyState::Origin)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.swordship_states,
                        state_id,
                        &mut swordship_occurrences,
                        SwordshipState::skill_id,
                    )
                    .map(MonsterPropertyState::Swordship)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.battle_fairy_attribute_states,
                        state_id,
                        &mut battle_fairy_occurrences,
                        BattleFairyAttributeState::skill_id,
                    )
                    .map(MonsterPropertyState::BattleFairyAttribute)
                })
                .or_else(|| {
                    next_ordered_state(
                        &self.fury_states,
                        state_id,
                        &mut fury_occurrences,
                        FuryState::skill_id,
                    )
                    .map(MonsterPropertyState::Fury)
                })
                .or_else(|| {
                    self.weak_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(MonsterPropertyState::Weak)
                })
                .or_else(|| {
                    self.poison_fog_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(MonsterPropertyState::PoisonFog)
                })
                .or_else(|| {
                    self.god_bless_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(MonsterPropertyState::GodBless)
                })
                .or_else(|| {
                    self.roar_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(MonsterPropertyState::Roar)
                })
                .or_else(|| {
                    self.boss_blue_fury_state
                        .filter(|state| state.skill_id() == state_id)
                        .map(MonsterPropertyState::BossBlueFury)
                });
            if let Some(state) = state {
                ordered.push(state);
            }
        }
        ordered
    }
    pub(crate) fn take_expired_god_bless_state(&mut self, now_ms: u32) -> Option<GodBlessState> {
        let state = self.god_bless_state.filter(|state| state.expired(now_ms))?;
        self.god_bless_state = None;
        self.remove_serialized_state_record(state.skill_id(), GOD_BLESS_STATE_BYTES);
        Some(state)
    }
    pub(crate) const fn soul_collect_state(&self) -> Option<SoulCollectState> {
        self.state_storage.soul_collect_state
    }

    pub(crate) fn begin_soul_collect_state(&mut self, state: SoulCollectState) {
        debug_assert!(self.soul_collect_state.is_none());
        self.remove_serialized_state_record(state.skill_id(), SOUL_COLLECT_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded());
        self.soul_collect_state = Some(state);
    }

    pub(crate) fn activate_loaded_soul_collect_state(&self) -> Option<SoulCollectState> { self.soul_collect_state }

    pub(crate) fn soul_collect_state_mut(&mut self) -> Option<&mut SoulCollectState> {
        self.soul_collect_state.as_mut()
    }

    pub(crate) fn take_soul_collect_state(&mut self) -> Option<SoulCollectState> {
        let state = self.soul_collect_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), SOUL_COLLECT_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_expired_spider_web_state(
        &mut self,
        now_ms: u32,
    ) -> Option<SpiderWebState> {
        let state = self.spider_web_state.filter(|state| state.expired(now_ms))?;
        self.spider_web_state = None;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), SPIDER_WEB_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_spider_web_state(&mut self) -> Option<SpiderWebState> {
        let state = self.spider_web_state.take()?;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), SPIDER_WEB_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_knock_out_state(&mut self, state: KnockOutState) -> Option<KnockOutState> {
        self.remove_serialized_state_record(state.skill_id(), KNOCK_OUT_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.blind_state_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.knock_out_state.replace(state)
    }

    pub(crate) fn activate_loaded_knock_out_state(&mut self, now_ms: u32) -> Option<KnockOutState> {
        let state = self.knock_out_state?.activate_loaded(now_ms);
        self.knock_out_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) fn activate_loaded_blind_state(&mut self, now_ms: u32) -> Option<BlindState> {
        let state = self.blind_state?.activate_loaded(now_ms);
        self.blind_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) fn take_expired_blind_state(&mut self, now_ms: u32) -> Option<BlindState> {
        self.blind_state.filter(|state| state.expired(now_ms))?;
        self.take_blind_state()
    }

    pub(crate) fn take_blind_state(&mut self) -> Option<BlindState> {
        let state = self.blind_state.take()?;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.remove_blind_state_serialized();
        Some(state)
    }

    fn remove_blind_state_serialized(&mut self) {
        let Some(offset) = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .find(|offset| read_u32(&self.ex_states, *offset) == Some(BLIND_STATE_ID))
        else {
            return;
        };
        let end = offset.saturating_add(BLIND_STATE_BYTES);
        if end > self.ex_states.len() {
            return;
        }
        self.ex_states.drain(offset..end);
        if self.ex_states.len() >= 4 {
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
        }
        for state in &mut self.extended_states {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        for state in &mut self.change_body_states {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        for state in &mut self.undead_states {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        if let Some(state) = &mut self.leaf_cut_state {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        if let Some(state) = &mut self.leaf_cut_3_state {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        if let Some(state) = &mut self.kerosene_state {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        if let Some(state) = &mut self.poison_fog_state {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        if let Some(state) = &mut self.meteor_arrow_state {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
        if let Some(state) = &mut self.ride_state {
            state.shift_serialized_offset_after(offset, BLIND_STATE_BYTES);
        }
    }

    pub(crate) fn replace_boa_lock_state(&mut self, state: BoaLockState) -> Option<BoaLockState> {
        self.remove_serialized_state_record(state.skill_id(), BOA_LOCK_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.curable_state_order.insert(state.skill_id());
        self.boa_lock_state.replace(state)
    }

    pub(crate) fn activate_loaded_boa_lock_state(&mut self, now_ms: u32) -> Option<BoaLockState> {
        let state = self.boa_lock_state?.activate_loaded(now_ms);
        self.boa_lock_state = Some(state);
        self.set_moveable(false);
        Some(state)
    }

    pub(crate) fn take_expired_boa_lock_state(&mut self, now_ms: u32) -> Option<BoaLockState> {
        self.boa_lock_state.filter(|state| state.expired(now_ms))?;
        self.take_boa_lock_state()
    }

    pub(crate) fn take_boa_lock_state(&mut self) -> Option<BoaLockState> {
        let state = self.boa_lock_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), BOA_LOCK_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn pillar_state(&self) -> Option<PillarState> { self.pillar_state }

    pub(crate) fn replace_rush_state(&mut self, state: RushState) -> Option<RushState> {
        self.remove_serialized_state_record(state.skill_id(), RUSH_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.curable_state_order.insert(state.skill_id());
        self.rush_state.replace(state)
    }

    pub(crate) fn activate_loaded_rush_state(&mut self, now_ms: u32) -> Option<RushState> {
        let state = self.rush_state?.activate_loaded(now_ms);
        self.rush_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) fn take_expired_rush_state(&mut self, now_ms: u32) -> Option<RushState> {
        self.rush_state.filter(|state| state.expired(now_ms))?;
        self.take_rush_state()
    }

    pub(crate) fn take_rush_state(&mut self) -> Option<RushState> {
        let state = self.rush_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), RUSH_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_rush_2_state(&mut self, state: Rush2State) -> Option<Rush2State> {
        self.remove_serialized_state_record(state.skill_id(), RUSH_2_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.curable_state_order.insert(state.skill_id());
        self.rush_2_state.replace(state)
    }

    pub(crate) fn activate_loaded_rush_2_state(&mut self, now_ms: u32) -> Option<Rush2State> {
        let state = self.rush_2_state?.activate_loaded(now_ms);
        self.rush_2_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) fn take_expired_rush_2_state(&mut self, now_ms: u32) -> Option<Rush2State> {
        self.rush_2_state.filter(|state| state.expired(now_ms))?;
        self.take_rush_2_state()
    }

    pub(crate) fn take_rush_2_state(&mut self) -> Option<Rush2State> {
        let state = self.rush_2_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), RUSH_2_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_pillar_state(&mut self, state: PillarState) -> Option<PillarState> {
        self.remove_serialized_state_record(state.skill_id(), PILLAR_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.pillar_state.replace(state)
    }

    pub(crate) fn activate_loaded_pillar_state(&mut self, now_ms: u32) -> Option<PillarState> {
        let state = self.pillar_state?.activate_loaded(now_ms);
        self.pillar_state = Some(state);
        self.set_moveable(false);
        Some(state)
    }
    pub(crate) fn take_expired_pillar_state(&mut self, now_ms: u32) -> Option<PillarState> {
        self.pillar_state.filter(|state| state.expired(now_ms))?;
        let state = self.pillar_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), PILLAR_STATE_BYTES);
        Some(state)
    }
    pub(crate) fn take_pillar_state(&mut self) -> Option<PillarState> {
        let state = self.pillar_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), PILLAR_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_expired_knock_out_state(&mut self, now_ms: u32) -> Option<KnockOutState> {
        let state = self.knock_out_state.filter(|state| state.expired(now_ms))?;
        self.knock_out_state = None;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), KNOCK_OUT_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_knock_out_state(&mut self) -> Option<KnockOutState> {
        let state = self.knock_out_state.take()?;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), KNOCK_OUT_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn replace_knight_cut_state(&mut self, state: KnightCutState) -> Option<KnightCutState> {
        self.remove_serialized_state_record(state.skill_id(), KNIGHT_CUT_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        self.curable_state_order.insert(state.skill_id());
        self.knight_cut_state.replace(state)
    }

    pub(crate) fn activate_loaded_knight_cut_state(&mut self, now_ms: u32) -> Option<KnightCutState> {
        let state = self.knight_cut_state?.activate_loaded(now_ms);
        self.knight_cut_state = Some(state);
        self.set_moveable(false);
        self.set_fightable(false);
        Some(state)
    }

    pub(crate) fn take_expired_knight_cut_state(&mut self, now_ms: u32) -> Option<KnightCutState> {
        let state = self.knight_cut_state.filter(|state| state.expired(now_ms))?;
        self.knight_cut_state = None;
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), KNIGHT_CUT_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn take_knight_cut_state(&mut self) -> Option<KnightCutState> {
        let state = self.knight_cut_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), KNIGHT_CUT_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn curable_state_ids(&self) -> Vec<u32> {
        self.curable_state_order.iter().copied().collect()
    }

    pub(crate) fn register_curable_skill_state(&mut self, skill_id: u32) {
        self.curable_state_order.shift_remove(&skill_id);
        self.curable_state_order.insert(skill_id);
    }

    pub(crate) fn finish_curable_skill_state(&mut self, skill_id: u32) {
        self.curable_state_order.shift_remove(&skill_id);
    }

    pub(crate) fn replace_poison_fog_state(&mut self, mut state: PoisonFogState, now_ms: u32) -> Option<PoisonFogState> {
        let previous = self.poison_fog_state.take();
        let replaced = previous.and_then(PoisonFogState::serialized_span).is_some_and(|(offset, amount)| amount == POISON_FOG_STATE_BYTES && state.write_serialized_at(&mut self.ex_states, offset, now_ms));
        if !replaced { if self.ex_states.len() < 4 { self.ex_states.clear(); LegacyWriter::new(&mut self.ex_states).write_u32(0); } let count = read_u32(&self.ex_states, 0).expect("счётчик состояний"); write_u32(&mut self.ex_states, 0, count.wrapping_add(1)); state.append_serialized(&mut self.ex_states, now_ms); }
        self.curable_state_order.shift_remove(&state.skill_id()); self.curable_state_order.insert(state.skill_id()); self.poison_fog_state = Some(state); previous
    }
    pub(crate) fn take_expired_poison_fog_state(&mut self, now_ms: u32) -> Option<PoisonFogState> { let state = self.poison_fog_state.filter(|state| state.expired(now_ms))?; self.finish_poison_fog_state(state); Some(state) }
    pub(crate) fn take_poison_fog_state(&mut self) -> Option<PoisonFogState> { let state = self.poison_fog_state?; self.finish_poison_fog_state(state); Some(state) }
    fn finish_poison_fog_state(&mut self, state: PoisonFogState) { self.poison_fog_state = None; self.curable_state_order.shift_remove(&state.skill_id()); let Some((offset, amount)) = state.serialized_span() else { return }; if offset.saturating_add(amount) > self.ex_states.len() { return } self.ex_states.drain(offset..offset + amount); if self.ex_states.len() >= 4 { let count = read_u32(&self.ex_states, 0).expect("счётчик состояний"); write_u32(&mut self.ex_states, 0, count.saturating_sub(1)); } for state in &mut self.extended_states { state.shift_serialized_offset_after(offset, amount); } for state in &mut self.change_body_states { state.shift_serialized_offset_after(offset, amount); } for state in &mut self.undead_states { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.leaf_cut_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.leaf_cut_3_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.kerosene_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.ride_state { state.shift_serialized_offset_after(offset, amount); } }
    pub(crate) fn activate_loaded_poison_fog_state(&mut self, now_ms: u32) -> Option<PoisonFogState> { let mut state = self.poison_fog_state?; state.activate_loaded(now_ms); state.update_serialized_runtime(&mut self.ex_states, now_ms); self.poison_fog_state = Some(state); Some(state) }

    pub(crate) fn meteor_arrow_state(&self) -> Option<MeteorArrowState> { self.meteor_arrow_state }
    pub(crate) fn add_meteor_arrows(&mut self, maximum: u32, amount: u32) -> Option<MeteorArrowState> {
        if let Some(mut state) = self.meteor_arrow_state {
            if !state.add_arrows(amount) { return None }
            state.update_serialized(&mut self.ex_states);
            self.meteor_arrow_state = Some(state);
            return Some(state);
        }
        let mut state = MeteorArrowState::new(maximum);
        if self.ex_states.len() < 4 { self.ex_states.clear(); LegacyWriter::new(&mut self.ex_states).write_u32(0); }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        state.append_serialized(&mut self.ex_states);
        self.meteor_arrow_state = Some(state);
        if !state.add_arrows(amount) { return None }
        state.update_serialized(&mut self.ex_states);
        self.meteor_arrow_state = Some(state);
        Some(state)
    }
    pub(crate) fn take_meteor_arrow_state(&mut self) -> Option<MeteorArrowState> {
        let state = self.meteor_arrow_state.take()?;
        let Some((offset, amount)) = state.serialized_span() else { return Some(state) };
        if offset.saturating_add(amount) > self.ex_states.len() { return Some(state) }
        self.ex_states.drain(offset..offset + amount);
        if self.ex_states.len() >= 4 { let count = read_u32(&self.ex_states, 0).expect("счётчик состояний"); write_u32(&mut self.ex_states, 0, count.saturating_sub(1)); }
        for known in &mut self.extended_states { known.shift_serialized_offset_after(offset, amount); }
        for known in &mut self.change_body_states { known.shift_serialized_offset_after(offset, amount); }
        for known in &mut self.undead_states { known.shift_serialized_offset_after(offset, amount); }
        if let Some(known) = &mut self.leaf_cut_state { known.shift_serialized_offset_after(offset, amount); }
        if let Some(known) = &mut self.leaf_cut_3_state { known.shift_serialized_offset_after(offset, amount); }
        if let Some(known) = &mut self.kerosene_state { known.shift_serialized_offset_after(offset, amount); }
        if let Some(known) = &mut self.poison_fog_state { known.shift_serialized_offset_after(offset, amount); }
        if let Some(known) = &mut self.ride_state { known.shift_serialized_offset_after(offset, amount); }
        Some(state)
    }

    pub(crate) fn blind_state_order(&self) -> Vec<u32> {
        self.blind_state_order.iter().copied().collect()
    }

    pub(crate) fn replace_blood_loss_state(
        &mut self,
        state: BloodLossState,
    ) -> Option<BloodLossState> {
        self.periodic_attack_order.insert(state.skill_id());
        let previous = self.blood_loss_state.replace(state);
        let serialized_exists = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .find(|offset| read_u32(&self.ex_states, *offset) == Some(state.skill_id()))
            .is_some();
        if previous.is_some() && serialized_exists {
            update_known_state_record(
                &mut self.ex_states,
                state.skill_id(),
                &state.encoded_for_install(),
            );
        } else if !serialized_exists {
            self.append_serialized_state_record(&state.encoded_for_install());
        }
        previous
    }

    pub(crate) fn finish_blood_loss_state(&mut self, state: BloodLossState) {
        self.periodic_attack_order.shift_remove(&state.skill_id());
        self.remove_serialized_state_record(state.skill_id(), BLOOD_LOSS_STATE_BYTES);
    }

    pub(crate) fn replace_leaf_cut_state(
        &mut self,
        mut state: LeafCutState,
        now_ms: u32,
    ) -> Option<LeafCutState> {
        let previous = self.leaf_cut_state.take();
        let replaced_in_place = previous
            .and_then(LeafCutState::serialized_span)
            .is_some_and(|(offset, amount)| {
                amount == LEAF_CUT_STATE_BYTES
                    && state.write_serialized_at(&mut self.ex_states, offset, now_ms)
            });
        if !replaced_in_place {
            if self.ex_states.len() < 4 {
                self.ex_states.clear();
                LegacyWriter::new(&mut self.ex_states).write_u32(0);
            }
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
            state.append_serialized(&mut self.ex_states, now_ms);
        }
        self.periodic_attack_order.insert(state.skill_id());
        self.leaf_cut_state = Some(state);
        previous
    }

    pub(crate) fn take_leaf_cut_state_for_ai(&mut self) -> Option<LeafCutState> {
        self.leaf_cut_state.take()
    }

    pub(crate) fn restore_leaf_cut_state_after_ai(&mut self, state: LeafCutState) {
        debug_assert!(self.leaf_cut_state.is_none());
        self.leaf_cut_state = Some(state);
    }

    pub(crate) fn finish_leaf_cut_state(&mut self, state: LeafCutState) {
        self.remove_leaf_cut_state_serialized(state);
        self.periodic_attack_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
    }

    fn remove_leaf_cut_state_serialized(&mut self, state: LeafCutState) {
        let Some((offset, amount)) = state.serialized_span() else { return };
        if offset.saturating_add(amount) > self.ex_states.len() { return }
        self.ex_states.drain(offset..offset + amount);
        if self.ex_states.len() >= 4 {
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
        }
        for state in &mut self.extended_states { state.shift_serialized_offset_after(offset, amount); }
        for state in &mut self.change_body_states { state.shift_serialized_offset_after(offset, amount); }
        for state in &mut self.undead_states { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.leaf_cut_3_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.kerosene_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.poison_fog_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.ride_state { state.shift_serialized_offset_after(offset, amount); }
    }

    pub(crate) fn activate_loaded_leaf_cut_state(&mut self, now_ms: u32) -> Option<LeafCutState> {
        let mut state = self.leaf_cut_state?;
        state.activate_loaded(now_ms);
        state.update_serialized_runtime(&mut self.ex_states, now_ms);
        self.leaf_cut_state = Some(state);
        Some(state)
    }

    pub(crate) fn replace_leaf_cut_2_state(
        &mut self,
        state: LeafCutState2,
    ) -> Option<LeafCutState2> {
        let serialized_exists = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .any(|offset| read_u32(&self.ex_states, offset) == Some(LEAF_CUT_2_STATE_ID));
        if serialized_exists {
            update_known_state_record(&mut self.ex_states, LEAF_CUT_2_STATE_ID, &state.encoded_for_install());
        } else {
            self.append_serialized_state_record(&state.encoded_for_install());
        }
        self.periodic_attack_order.insert(LEAF_CUT_2_STATE_ID);
        self.leaf_cut_2_state.replace(state)
    }

    pub(crate) fn take_leaf_cut_2_state_for_ai(&mut self) -> Option<LeafCutState2> {
        self.leaf_cut_2_state.take()
    }

    pub(crate) fn restore_leaf_cut_2_state_after_ai(&mut self, state: LeafCutState2) {
        debug_assert!(self.leaf_cut_2_state.is_none());
        self.leaf_cut_2_state = Some(state);
    }

    pub(crate) fn finish_leaf_cut_2_state(&mut self) {
        self.leaf_cut_2_state = None;
        self.periodic_attack_order.shift_remove(&LEAF_CUT_2_STATE_ID);
        self.curable_state_order.shift_remove(&LEAF_CUT_2_STATE_ID);
        self.remove_serialized_state_record(LEAF_CUT_2_STATE_ID, LEAF_CUT_2_STATE_BYTES);
    }

    pub(crate) fn activate_loaded_leaf_cut_2_state(&mut self, now_ms: u32) -> Option<LeafCutState2> {
        let mut state = self.leaf_cut_2_state?;
        state.activate_loaded(now_ms);
        update_known_state_record(&mut self.ex_states, LEAF_CUT_2_STATE_ID, &state.encoded(now_ms));
        self.leaf_cut_2_state = Some(state);
        Some(state)
    }

    pub(crate) fn replace_leaf_cut_3_state(
        &mut self,
        mut state: LeafCutState3,
        now_ms: u32,
    ) -> Option<LeafCutState3> {
        let previous = self.leaf_cut_3_state.take();
        let replaced_in_place = previous
            .and_then(LeafCutState3::serialized_span)
            .is_some_and(|(offset, amount)| {
                amount == LEAF_CUT_3_STATE_BYTES
                    && state.write_serialized_at(&mut self.ex_states, offset, now_ms)
            });
        if !replaced_in_place {
            if self.ex_states.len() < 4 {
                self.ex_states.clear();
                LegacyWriter::new(&mut self.ex_states).write_u32(0);
            }
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
            state.append_serialized(&mut self.ex_states, now_ms);
        }
        self.periodic_attack_order.insert(state.skill_id());
        self.leaf_cut_3_state = Some(state);
        previous
    }

    pub(crate) fn take_leaf_cut_3_state_for_ai(&mut self) -> Option<LeafCutState3> {
        self.leaf_cut_3_state.take()
    }

    pub(crate) fn restore_leaf_cut_3_state_after_ai(&mut self, state: LeafCutState3) {
        debug_assert!(self.leaf_cut_3_state.is_none());
        self.leaf_cut_3_state = Some(state);
    }

    pub(crate) fn finish_leaf_cut_3_state(&mut self, state: LeafCutState3) {
        self.remove_leaf_cut_3_state_serialized(state);
        self.periodic_attack_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
    }

    fn remove_leaf_cut_3_state_serialized(&mut self, state: LeafCutState3) {
        let Some((offset, amount)) = state.serialized_span() else { return };
        if offset.saturating_add(amount) > self.ex_states.len() { return }
        self.ex_states.drain(offset..offset + amount);
        if self.ex_states.len() >= 4 {
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
        }
        for state in &mut self.extended_states { state.shift_serialized_offset_after(offset, amount); }
        for state in &mut self.change_body_states { state.shift_serialized_offset_after(offset, amount); }
        for state in &mut self.undead_states { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.leaf_cut_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.kerosene_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.poison_fog_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.ride_state { state.shift_serialized_offset_after(offset, amount); }
    }

    pub(crate) fn activate_loaded_leaf_cut_3_state(&mut self, now_ms: u32) -> Option<LeafCutState3> {
        let mut state = self.leaf_cut_3_state?;
        state.activate_loaded(now_ms);
        state.update_serialized_runtime(&mut self.ex_states, now_ms);
        self.leaf_cut_3_state = Some(state);
        Some(state)
    }

    pub(crate) fn activate_loaded_strike_states(&mut self, now_ms: u32) -> Vec<StrikeState> {
        for state in &mut self.strike_states { *state = state.activate_loaded(now_ms); }
        for _ in 0..self.strike_states.len() { self.set_moveable(false); self.set_fightable(false); }
        self.strike_states.clone()
    }

    pub(crate) fn take_expired_strike_states(&mut self, now_ms: u32) -> Vec<StrikeState> {
        let mut ended = Vec::new();
        let mut position = 0usize;
        while position < self.strike_states.len() {
            if !self.strike_states[position].expired(now_ms) { position += 1; continue; }
            let state = self.strike_states.remove(position);
            if let Some(offset) = known_state_record_offsets(&self.ex_states).into_iter().filter(|offset| read_u32(&self.ex_states, *offset) == Some(STRIKE_STATE_ID)).nth(position) {
                self.remove_serialized_state_record_at(offset, STRIKE_STATE_BYTES);
            }
            self.set_moveable(true); self.set_fightable(true); ended.push(state);
        }
        if self.strike_states.is_empty() { self.blind_state_order.shift_remove(&STRIKE_STATE_ID); }
        ended
    }

    pub(crate) fn take_strike_states(&mut self) -> Vec<StrikeState> {
        let ended = std::mem::take(&mut self.strike_states);
        self.blind_state_order.shift_remove(&STRIKE_STATE_ID);
        for _ in 0..ended.len() {
            self.remove_serialized_state_record(STRIKE_STATE_ID, STRIKE_STATE_BYTES);
            self.set_moveable(true);
            self.set_fightable(true);
        }
        ended
    }

    pub(crate) fn kerosene_state(&self) -> Option<KeroseneState> { self.kerosene_state }
    pub(crate) fn replace_kerosene_state(&mut self, mut state: KeroseneState, now_ms: u32) -> Option<KeroseneState> {
        let previous = self.kerosene_state.take();
        let replaced = previous.and_then(KeroseneState::serialized_span).is_some_and(|(offset, amount)| amount == KEROSENE_STATE_BYTES && state.write_serialized_at(&mut self.ex_states, offset, now_ms));
        if !replaced { if self.ex_states.len() < 4 { self.ex_states.clear(); LegacyWriter::new(&mut self.ex_states).write_u32(0); } let count = read_u32(&self.ex_states, 0).expect("счётчик состояний"); write_u32(&mut self.ex_states, 0, count.wrapping_add(1)); state.append_serialized(&mut self.ex_states, now_ms); }
        self.periodic_attack_order.insert(state.skill_id()); self.kerosene_state = Some(state); previous
    }
    pub(crate) fn take_kerosene_state_for_ai(&mut self) -> Option<KeroseneState> { self.kerosene_state.take() }
    pub(crate) fn restore_kerosene_state_after_ai(&mut self, state: KeroseneState) { debug_assert!(self.kerosene_state.is_none()); self.kerosene_state = Some(state); }
    pub(crate) fn take_kerosene_state(&mut self) -> Option<KeroseneState> { let state = self.kerosene_state.take()?; self.finish_kerosene_state(state); Some(state) }
    pub(crate) fn finish_kerosene_state(&mut self, state: KeroseneState) {
        self.periodic_attack_order.shift_remove(&state.skill_id());
        let Some((offset, amount)) = state.serialized_span() else { return }; if offset.saturating_add(amount) > self.ex_states.len() { return }
        self.ex_states.drain(offset..offset + amount); if self.ex_states.len() >= 4 { let count = read_u32(&self.ex_states, 0).expect("счётчик состояний"); write_u32(&mut self.ex_states, 0, count.saturating_sub(1)); }
        for known in &mut self.extended_states { known.shift_serialized_offset_after(offset, amount); } for known in &mut self.change_body_states { known.shift_serialized_offset_after(offset, amount); } for known in &mut self.undead_states { known.shift_serialized_offset_after(offset, amount); } if let Some(known) = &mut self.leaf_cut_state { known.shift_serialized_offset_after(offset, amount); } if let Some(known) = &mut self.leaf_cut_3_state { known.shift_serialized_offset_after(offset, amount); } if let Some(known) = &mut self.poison_fog_state { known.shift_serialized_offset_after(offset, amount); } if let Some(known) = &mut self.meteor_arrow_state { known.shift_serialized_offset_after(offset, amount); } if let Some(known) = &mut self.ride_state { known.shift_serialized_offset_after(offset, amount); }
    }
    pub(crate) fn activate_loaded_kerosene_state(&mut self, now_ms: u32) -> Option<KeroseneState> { let mut state = self.kerosene_state?; state.activate_loaded(now_ms); state.update_serialized_runtime(&mut self.ex_states, now_ms); self.kerosene_state = Some(state); Some(state) }

    pub(crate) fn battle_fairy_attribute_states(&self) -> &[BattleFairyAttributeState] {
        &self.battle_fairy_attribute_states
    }

    pub(crate) fn replace_battle_fairy_attribute_state(
        &mut self,
        state: BattleFairyAttributeState,
    ) -> Option<BattleFairyAttributeState> {
        self.remove_serialized_state_record(state.skill_id(), BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        let previous = self
            .battle_fairy_attribute_states
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
            .map(|position| self.battle_fairy_attribute_states.remove(position));
        self.battle_fairy_attribute_states.push(state);
        previous
    }

    pub(crate) fn take_expired_battle_fairy_attribute_states(
        &mut self,
        now_ms: u32,
    ) -> Vec<BattleFairyAttributeState> {
        let mut expired = Vec::new();
        let mut position = 0;
        while position < self.battle_fairy_attribute_states.len() {
            if self.battle_fairy_attribute_states[position].expired(now_ms) {
                let state = self.battle_fairy_attribute_states.remove(position);
                self.remove_serialized_state_record(state.skill_id(), BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES);
                expired.push(state);
            } else {
                position += 1;
            }
        }
        expired
    }

    pub(crate) fn activate_loaded_battle_fairy_attribute_states(&mut self, now_ms: u32) -> Vec<BattleFairyAttributeState> {
        for state in &mut self.battle_fairy_attribute_states { *state = state.activate_loaded(now_ms); }
        self.battle_fairy_attribute_states.clone()
    }

    pub(crate) fn take_blood_loss_state_for_ai(&mut self) -> Option<BloodLossState> {
        self.blood_loss_state.take()
    }

    pub(crate) fn activate_loaded_blood_loss_state(&mut self, now_ms: u32) -> Option<BloodLossState> {
        let mut state = self.blood_loss_state?;
        state.activate_loaded(now_ms);
        self.blood_loss_state = Some(state);
        Some(state)
    }

    pub(crate) fn periodic_attack_state_ids(&self) -> Vec<u32> {
        self.periodic_attack_order.iter().copied().collect()
    }

    pub(crate) fn finish_periodic_attack_state(&mut self, skill_id: u32) {
        self.periodic_attack_order.shift_remove(&skill_id);
        self.curable_state_order.shift_remove(&skill_id);
    }

    pub(crate) fn agility_state(&self, skill_id: u32) -> Option<AgilityState> {
        match self.persistent_agility_family_state {
            Some(PersistentAgilityFamilyState::Agility(state)) if state.skill_id() == skill_id => {
                Some(state)
            }
            _ => None,
        }
    }

    pub(crate) fn take_agility_state(&mut self, skill_id: u32) -> Option<AgilityState> {
        match self.persistent_agility_family_state {
            Some(PersistentAgilityFamilyState::Agility(state)) if state.skill_id() == skill_id => {
                self.persistent_agility_family_state.take();
                self.remove_serialized_state_record(skill_id, PERSISTENT_AGILITY_FAMILY_STATE_BYTES);
                Some(state)
            }
            _ => None,
        }
    }

    pub(crate) fn begin_agility_state(&mut self, state: AgilityState) {
        debug_assert_eq!(
            state.skill_id(),
            crate::gameserver::appserver::skills::agility::AGILITY_SKILL_ID
        );
        debug_assert!(self.persistent_agility_family_state.is_none());
        self.append_serialized_state_record(&PersistentAgilityFamilyState::Agility(state).encoded());
        self.persistent_agility_family_state =
            Some(PersistentAgilityFamilyState::Agility(state));
    }

    pub(crate) fn agility_state_2(&self) -> Option<AgilityState2> {
        self.agility_state_2
    }

    pub(crate) fn take_agility_state_2(&mut self) -> Option<AgilityState2> {
        let state = self.agility_state_2.take()?;
        self.remove_serialized_state_record(state.skill_id(), AGILITY_STATE_2_BYTES);
        Some(state)
    }

    pub(crate) fn begin_agility_state_2(&mut self, state: AgilityState2) {
        debug_assert!(self.agility_state_2.is_none());
        self.append_serialized_state_record(&state.encoded_for_install());
        self.agility_state_2 = Some(state);
    }

    pub(crate) fn persistent_agility_family_state(
        &self,
    ) -> Option<PersistentAgilityFamilyState> {
        self.persistent_agility_family_state
    }

    pub(crate) fn take_persistent_agility_family_state(
        &mut self,
    ) -> Option<PersistentAgilityFamilyState> {
        let state = self.persistent_agility_family_state.take()?;
        self.remove_serialized_state_record(state.skill_id(), PERSISTENT_AGILITY_FAMILY_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn begin_persistent_agility_family_state(
        &mut self,
        state: PersistentAgilityFamilyState,
    ) {
        debug_assert!(PersistentAgilityFamilyState::is_known_skill(
            state.skill_id()
        ));
        debug_assert!(self.persistent_agility_family_state.is_none());
        self.append_serialized_state_record(&state.encoded());
        self.persistent_agility_family_state = Some(state);
    }

    pub(crate) fn take_expired_agility_state_2(&mut self, now_ms: u32) -> Option<AgilityState2> {
        self.agility_state_2.filter(|state| state.expired(now_ms))?;
        self.take_agility_state_2()
    }

    pub(crate) fn activate_loaded_agility_states(
        &mut self,
        now_ms: u32,
    ) -> (Option<PersistentAgilityFamilyState>, Option<AgilityState2>) {
        if let Some(state) = &mut self.agility_state_2 {
            state.activate_loaded(now_ms);
        }
        (self.persistent_agility_family_state, self.agility_state_2)
    }

    pub(crate) fn take_first_script_state(&mut self, state_id: i32) -> Option<ScriptMoveState> {
        let index = self
            .script_states
            .iter()
            .position(|state| state.state_id() == state_id)?;
        self.remove_script_state_at(index)
    }

    pub(crate) fn script_states(&self) -> &[ScriptMoveState] {
        &self.script_states
    }

    pub(crate) fn script_state(&self, index: usize) -> Option<ScriptMoveState> {
        self.script_states.get(index).copied()
    }

    pub(crate) fn remove_script_state_at(&mut self, index: usize) -> Option<ScriptMoveState> {
        let state = self.script_states.get(index).copied()?;
        let occurrence = self.script_states[..index]
            .iter()
            .filter(|candidate| candidate.state_id() == state.state_id())
            .count();
        let offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .filter(|offset| read_u32(&self.ex_states, *offset) == Some(state.state_id() as u32))
            .nth(occurrence);
        let removed = self.script_states.remove(index);
        if let Some(offset) = offset
            && let Some(size) = ScriptMoveState::serialized_size(state.state_id())
        {
            self.remove_serialized_state_record_at(offset, size);
        }
        Some(removed)
    }

    pub(crate) fn activate_loaded_script_states(&mut self, now_ms: u32) -> Vec<ScriptMoveState> {
        for state in &mut self.script_states {
            state.activate_loaded(now_ms);
        }
        self.script_states.clone()
    }

    pub(crate) fn take_pending_script_state_visuals(&mut self) -> Vec<ScriptMoveState> {
        let mut pending = Vec::new();
        for state in &mut self.script_states {
            if state.take_pending_visual() {
                pending.push(*state);
            }
        }
        pending
    }

    pub(crate) const fn tian_shen_xia_fan_state(&self) -> Option<TianShenXiaFanState> {
        self.state_storage.tian_shen_xia_fan_state
    }

    pub(crate) fn activate_loaded_tian_shen_xia_fan_state(
        &mut self,
        now_ms: u32,
    ) -> Option<TianShenXiaFanState> {
        let state = self.tian_shen_xia_fan_state?.activate_loaded(now_ms);
        self.tian_shen_xia_fan_state = Some(state);
        Some(state)
    }

    pub(crate) fn take_expired_tian_shen_xia_fan_state(
        &mut self,
        now_ms: u32,
    ) -> Option<TianShenXiaFanState> {
        let state = self.tian_shen_xia_fan_state.filter(|state| state.expired(now_ms))?;
        self.tian_shen_xia_fan_state = None;
        self.remove_serialized_state_record(state.state_id(), TIAN_SHEN_XIA_FAN_STATE_BYTES);
        Some(state)
    }

    pub(crate) const fn wangsheng_state(&self) -> Option<WangshengState> {
        self.state_storage.wangsheng_state
    }

    pub(crate) fn activate_loaded_wangsheng_state(
        &mut self,
        now_ms: u32,
    ) -> Option<WangshengState> {
        let state = self.wangsheng_state?.activate_loaded(now_ms);
        self.wangsheng_state = Some(state);
        Some(state)
    }

    pub(crate) fn take_expired_wangsheng_state(&mut self, now_ms: u32) -> Option<WangshengState> {
        let state = self.wangsheng_state.filter(|state| state.expired(now_ms))?;
        self.wangsheng_state = None;
        self.remove_serialized_state_record(state.state_id(), WANGSHENG_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn begin_ride_state(&mut self, mut state: RideState) -> Option<RideState> {
        if self.ride_state.is_some() {
            return None;
        }
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        state.append_serialized(&mut self.ex_states);
        self.set_fightable(false);
        self.ride_state = Some(state.clone());
        Some(state)
    }

    pub(crate) fn end_ride_state(&mut self) -> Option<RideState> {
        let state = self.ride_state.take()?;
        if let Some((offset, amount)) = state.serialized_span()
            && offset + amount <= self.ex_states.len()
        {
            self.ex_states.drain(offset..offset + amount);
            if self.ex_states.len() >= 4 {
                let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
                write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
            }
            for state in &mut self.extended_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.change_body_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.undead_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.leaf_cut_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.leaf_cut_3_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.kerosene_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.poison_fog_state { state.shift_serialized_offset_after(offset, amount); }
            if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); }
        }
        self.set_fightable(true);
        Some(state)
    }

    pub(crate) fn activate_loaded_ride_state(&mut self) -> Option<RideState> {
        self.ride_state.as_ref()?;
        self.set_fightable(false);
        self.ride_state.clone()
    }

    /// Exact `AddUndeadState`: registry key `(56, stateID)`, затем удаление
    /// всех state того же type либо ID, после чего ID `0` оставляет только
    /// removal tail. Успешный Begin хранит state и возвращает `1`.
    pub(crate) fn add_undead_state<Now>(
        &mut self,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: Now,
    ) -> UndeadStateMutation
    where
        Now: FnOnce() -> u32,
    {
        let now_ms = now_ms();
        let Some(mut state) = UndeadState::from_factory(state_id, factory, now_ms) else {
            return UndeadStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
                state_list_changed: false,
            };
        };
        let state_type = state.state_type;
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.undead_states.len() {
            if self.undead_states[index].state_type == state_type
                || self.undead_states[index].state_id == state_id
            {
                let removed_state = self.undead_states.remove(index);
                self.remove_undead_state_serialized(&removed_state);
                removed.push(removed_state);
            } else {
                index += 1;
            }
        }
        if state_id == 0 {
            return UndeadStateMutation {
                state_list_changed: !removed.is_empty(),
                removed,
                added: None,
                legacy_return: 0,
            };
        }
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        let offset = self.ex_states.len();
        self.ex_states
            .resize(offset + 4 + UNDEAD_STATE_PARAMETER_BYTES, 0);
        state.write_serialized(&mut self.ex_states, offset);
        self.undead_states.push(state.clone());
        UndeadStateMutation {
            removed,
            added: Some(state),
            legacy_return: 1,
            state_list_changed: true,
        }
    }

    /// Exact first-match `DelUndeadState`; native `End` удаляет найденный
    /// state и возвращает его ID, отсутствующий state возвращает ноль.
    pub(crate) fn delete_undead_state(&mut self, state_id: u32) -> UndeadStateMutation {
        let Some(index) = self
            .undead_states
            .iter()
            .position(|state| state.state_id == state_id)
        else {
            return UndeadStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
                state_list_changed: false,
            };
        };
        let removed = self.undead_states.remove(index);
        self.remove_undead_state_serialized(&removed);
        UndeadStateMutation {
            removed: vec![removed],
            added: None,
            legacy_return: state_id,
            state_list_changed: true,
        }
    }

    pub(crate) fn get_undead_state(&self, state_id: u32) -> u32 {
        self.undead_states
            .iter()
            .any(|state| state.state_id == state_id)
            .then_some(state_id)
            .unwrap_or(0)
    }

    fn remove_undead_state_serialized(&mut self, state: &UndeadState) {
        let Some((offset, amount)) = state.serialized_span() else {
            return;
        };
        if offset + amount > self.ex_states.len() {
            return;
        }
        self.ex_states.drain(offset..offset + amount);
        if self.ex_states.len() >= 4 {
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
        }
        for state in &mut self.extended_states {
            state.shift_serialized_offset_after(offset, amount);
        }
        for state in &mut self.change_body_states {
            state.shift_serialized_offset_after(offset, amount);
        }
        for state in &mut self.undead_states {
            state.shift_serialized_offset_after(offset, amount);
        }
        if let Some(state) = &mut self.leaf_cut_state {
            state.shift_serialized_offset_after(offset, amount);
        }
        if let Some(state) = &mut self.leaf_cut_3_state {
            state.shift_serialized_offset_after(offset, amount);
        }
        if let Some(state) = &mut self.kerosene_state {
            state.shift_serialized_offset_after(offset, amount);
        }
        if let Some(state) = &mut self.poison_fog_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); }
        if let Some(state) = &mut self.ride_state {
            state.shift_serialized_offset_after(offset, amount);
        }
    }

    pub(crate) fn activate_loaded_undead_states(&mut self, now_ms: u32) -> Vec<UndeadState> {
        let storage = &mut self.state_storage;
        for state in &mut storage.undead_states {
            state.started_ms = now_ms;
            state.last_item_tick_ms = now_ms;
            state.update_serialized_runtime(&mut storage.ex_states, now_ms);
        }
        storage.undead_states.clone()
    }

    pub(crate) fn undead_state_tick(
        &mut self,
        now_ms: u32,
        dead: bool,
    ) -> (Vec<u32>, Vec<(u32, u32, u32)>) {
        let mut ended = Vec::new();
        let mut item_due = Vec::new();
        for state in &mut self.undead_states {
            if state.expired(now_ms) || (dead && state.disappear_after_dead) {
                ended.push(state.state_id);
                continue;
            }
            if state.item_due(now_ms) {
                state.last_item_tick_ms = now_ms;
                item_due.push((state.state_id, state.item_index, state.item_amount));
            }
        }
        (ended, item_due)
    }

    pub(crate) fn add_extended_state(
        &mut self,
        kind: ExtendedStateKind,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: u32,
    ) -> ExtendedStateMutation {
        let Some(mut added) = ExtendedState::from_factory(kind, state_id, factory, now_ms) else {
            return ExtendedStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
            };
        };
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.extended_states.len() {
            if self.extended_states[index].kind == kind
                && (self.extended_states[index].state_type == added.state_type
                    || self.extended_states[index].level == state_id)
            {
                let state = self.extended_states.remove(index);
                self.remove_extended_state_serialized(&state);
                removed.push(state);
            } else {
                index += 1;
            }
        }
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        let offset = self.ex_states.len();
        let size = match kind {
            ExtendedStateKind::Original => 44,
            ExtendedStateKind::New => 56,
        };
        self.ex_states.resize(offset + size, 0);
        added.write_serialized(&mut self.ex_states, offset);
        self.extended_states.push(added.clone());
        ExtendedStateMutation {
            removed,
            added: Some(added),
            legacy_return: 1,
        }
    }

    pub(crate) fn delete_extended_state(
        &mut self,
        kind: ExtendedStateKind,
        state_id: u32,
    ) -> ExtendedStateMutation {
        let Some(index) = self
            .extended_states
            .iter()
            .position(|state| state.kind == kind && state.level == state_id)
        else {
            return ExtendedStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
            };
        };
        let removed = self.extended_states.remove(index);
        self.remove_extended_state_serialized(&removed);
        ExtendedStateMutation {
            removed: vec![removed],
            added: None,
            legacy_return: state_id,
        }
    }

    pub(crate) fn delete_extended_state_by_type(
        &mut self,
        state_type: u16,
    ) -> ExtendedStateMutation {
        let Some(index) = self.extended_states.iter().position(|state| {
            state.kind == ExtendedStateKind::Original && state.state_type == state_type
        }) else {
            return ExtendedStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
            };
        };
        let removed = self.extended_states.remove(index);
        let legacy_return = removed.level;
        self.remove_extended_state_serialized(&removed);
        ExtendedStateMutation {
            removed: vec![removed],
            added: None,
            legacy_return,
        }
    }

    fn remove_extended_state_serialized(&mut self, state: &ExtendedState) {
        let span = state.serialized_span();
        state.remove_serialized(&mut self.ex_states);
        if let Some((offset, amount)) = span {
            for state in &mut self.extended_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.change_body_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.undead_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.leaf_cut_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.leaf_cut_3_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.kerosene_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.poison_fog_state { state.shift_serialized_offset_after(offset, amount); }
            if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); }
            if let Some(state) = &mut self.ride_state {
                state.shift_serialized_offset_after(offset, amount);
            }
        }
    }

    pub(crate) fn get_extended_state(&self, kind: ExtendedStateKind, state_id: u32) -> u32 {
        self.extended_states
            .iter()
            .any(|state| state.kind == kind && state.level == state_id)
            .then_some(state_id)
            .unwrap_or(0)
    }

    pub(crate) fn extended_states(&self) -> &[ExtendedState] {
        &self.extended_states
    }

    pub(crate) fn activate_loaded_extended_states(&mut self, now_ms: u32) -> Vec<ExtendedState> {
        let storage = &mut self.state_storage;
        for state in &mut storage.extended_states {
            state.activate_loaded(now_ms);
            state.update_serialized_runtime(&mut storage.ex_states, now_ms);
        }
        storage.extended_states.clone()
    }

    pub(crate) fn extended_state_tick(
        &mut self,
        now_ms: u32,
    ) -> (
        Vec<(ExtendedStateKind, u32)>,
        Vec<(ExtendedStateKind, u32, u32, u32)>,
    ) {
        let mut expired = Vec::new();
        let mut item_due = Vec::new();
        for state in &mut self.extended_states {
            if state.expired(now_ms) {
                expired.push((state.kind, state.level));
                continue;
            }
            if state.item_due(now_ms) {
                state.restart_item_clock(now_ms);
                item_due.push((state.kind, state.level, state.item_index, state.item_amount));
            }
        }
        (expired, item_due)
    }

    pub(crate) fn add_change_body_state(
        &mut self,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: u32,
        old_hotkeys: [u32; 12],
    ) -> ChangeBodyMutation {
        let Some(mut added) = ChangeBodyState::from_factory(state_id, factory, now_ms) else {
            return ChangeBodyMutation {
                removed: None,
                added: None,
                legacy_return: 0,
            };
        };
        added.old_hotkeys = old_hotkeys;
        let removed = self
            .change_body_states
            .iter()
            .position(|state| state.level == state_id)
            .map(|index| {
                let removed = self.change_body_states.remove(index);
                self.remove_change_body_state_serialized(&removed);
                removed
            });
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        let offset = self.ex_states.len();
        LegacyWriter::new(&mut self.ex_states).write_u32(super::chbystate::CHANGE_BODY_STATE_ID);
        self.ex_states.resize(offset + 124, 0);
        added.write_serialized(&mut self.ex_states, offset);
        self.change_body_states.push(added.clone());
        ChangeBodyMutation {
            removed,
            added: Some(added),
            legacy_return: 1,
        }
    }

    pub(crate) fn delete_change_body_state(&mut self, state_id: u32) -> ChangeBodyMutation {
        let Some(index) = self
            .change_body_states
            .iter()
            .position(|state| state.level == state_id)
        else {
            return ChangeBodyMutation {
                removed: None,
                added: None,
                legacy_return: 0,
            };
        };
        let removed = self.change_body_states.remove(index);
        self.remove_change_body_state_serialized(&removed);
        ChangeBodyMutation {
            removed: Some(removed),
            added: None,
            legacy_return: state_id,
        }
    }

    pub(crate) fn get_change_body_state(&self, state_id: u32) -> u32 {
        self.change_body_states
            .iter()
            .any(|state| state.level == state_id)
            .then_some(state_id)
            .unwrap_or_default()
    }

    fn remove_change_body_state_serialized(&mut self, state: &ChangeBodyState) {
        let span = state.serialized_span();
        state.remove_serialized(&mut self.ex_states);
        if let Some((offset, amount)) = span {
            for state in &mut self.extended_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.change_body_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.undead_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.leaf_cut_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.leaf_cut_3_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.kerosene_state {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.poison_fog_state { state.shift_serialized_offset_after(offset, amount); }
            if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); }
            if let Some(state) = &mut self.ride_state {
                state.shift_serialized_offset_after(offset, amount);
            }
        }
    }

    pub(crate) fn active_change_body_state(&self) -> Option<&ChangeBodyState> {
        self.change_body_states.last()
    }

    pub(crate) fn first_change_body_state_id(&self) -> Option<u32> {
        self.change_body_states.first().map(|state| state.level)
    }

    pub(crate) fn activate_loaded_change_body_states(
        &mut self,
        now_ms: u32,
    ) -> Vec<ChangeBodyState> {
        let storage = &mut self.state_storage;
        for state in &mut storage.change_body_states {
            state.activate_loaded(now_ms);
            state.update_serialized_runtime(&mut storage.ex_states, now_ms);
        }
        storage.change_body_states.clone()
    }

    pub(crate) fn expired_change_body_state_ids(&self, now_ms: u32) -> Vec<u32> {
        self.change_body_states
            .iter()
            .filter(|state| state.expired(now_ms))
            .map(|state| state.level)
            .collect()
    }

    pub(crate) fn change_body_region_transition_end_ids(&mut self) -> Vec<u32> {
        let mut ended = Vec::new();
        let storage = &mut self.state_storage;
        for state in &mut storage.change_body_states {
            if state.on_change_region() {
                ended.push(state.level);
            } else {
                state.update_serialized_runtime(&mut storage.ex_states, state.started_ms);
            }
        }
        ended
    }

    pub(crate) fn change_body_player_lost_end_ids(&mut self) -> Vec<u32> {
        let mut ended = Vec::new();
        let storage = &mut self.state_storage;
        for state in &mut storage.change_body_states {
            if state.on_player_lost() {
                ended.push(state.level);
            } else {
                state.update_serialized_runtime(&mut storage.ex_states, state.started_ms);
            }
        }
        ended
    }

    pub(crate) fn change_body_death_end_ids(&self) -> Vec<u32> {
        self.change_body_states
            .iter()
            .filter(|state| !state.continue_after_death)
            .map(|state| state.level)
            .collect()
    }

    pub(crate) fn skill(&self, skill_id: u32, factory: &CSkillFactory) -> Option<&MoveShapeSkill> {
        let category = SkillCategory::from_raw(factory.query_skill_type(skill_id, 1))?;
        self.skills_in_category(category).iter().find(|skill| skill.id == skill_id)
    }

    fn skill_mut(&mut self, skill_id: u32, factory: &CSkillFactory) -> Option<&mut MoveShapeSkill> {
        let category = SkillCategory::from_raw(factory.query_skill_type(skill_id, 1))?;
        self.skills[category as usize].iter_mut().find(|skill| skill.id == skill_id)
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
        mut execution: PlayerSkillExecution,
        factory: &CSkillFactory,
    ) -> bool {
        let skill_id = execution.kernel().dispatch().skill_id();
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        if !matches!(skill.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::Player(_)) {
            return false;
        }
        let lifecycle = std::mem::take(skill.execution.lifecycle_mut());
        execution.kernel_mut().replace_lifecycle(lifecycle);
        skill.execution = RegisteredSkillExecution::Player(execution);
        true
    }

    pub(crate) fn clear_player_execution(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        if !matches!(skill.execution, RegisteredSkillExecution::Player(_)) {
            return false;
        }
        let lifecycle = std::mem::take(skill.execution.lifecycle_mut());
        skill.execution = RegisteredSkillExecution::Inactive(lifecycle);
        true
    }

    pub(crate) fn battle_fairy_execution(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&BattleFairyExecution> {
        match &self.skill(skill_id, factory)?.execution {
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution),
            _ => None,
        }
    }

    pub(crate) fn battle_fairy_execution_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut BattleFairyExecution> {
        match &mut self.skill_mut(skill_id, factory)?.execution {
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution),
            _ => None,
        }
    }

    pub(crate) fn install_battle_fairy_execution(
        &mut self,
        mut execution: BattleFairyExecution,
        factory: &CSkillFactory,
    ) -> bool {
        let skill_id = execution.kernel().dispatch().skill_id();
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        if !matches!(skill.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::BattleFairy(_)) {
            return false;
        }
        let lifecycle = std::mem::take(skill.execution.lifecycle_mut());
        execution.kernel_mut().replace_lifecycle(lifecycle);
        skill.execution = RegisteredSkillExecution::BattleFairy(execution);
        true
    }

    pub(crate) fn clear_battle_fairy_execution(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        if !matches!(skill.execution, RegisteredSkillExecution::BattleFairy(_)) {
            return false;
        }
        let lifecycle = std::mem::take(skill.execution.lifecycle_mut());
        skill.execution = RegisteredSkillExecution::Inactive(lifecycle);
        true
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
        self.skills[SkillCategory::Defense as usize].push(MoveShapeSkill {
            id: SKILL_BASE_DEFENSE,
            level: 1,
            owner: CSkillFactory::factory_owner(SKILL_BASE_DEFENSE)
                .expect("CFightDefense входит в native factory"),
            item_position: -1,
            immediate_lifecycle: ImmediateSkillLifecycle::Unbegun,
            execution: RegisteredSkillExecution::Inactive(SkillLifecycle::default()),
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
        if self.skills_in_category(SkillCategory::Attack).iter().any(|skill| skill.id == 2) {
            2
        } else if self.skills_in_category(SkillCategory::Summon).iter().any(|skill| skill.id == 3) {
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
        let Some(owner) = CSkillFactory::factory_owner(skill_id) else {
            return false;
        };
        self.skills[owner.category() as usize].push(MoveShapeSkill {
            id: skill_id,
            level,
            owner,
            item_position: -1,
            immediate_lifecycle: ImmediateSkillLifecycle::Unbegun,
            execution: RegisteredSkillExecution::Inactive(SkillLifecycle::default()),
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
        let skills = &mut self.skills[category as usize];
        if let Some(index) = skills.iter().position(|skill| skill.id == skill_id) {
            skills.remove(index);
        }
        true
    }

    pub(crate) fn set_pos_xy(
        &mut self,
        region: &mut CRegion,
        x: f32,
        y: f32,
        facts: MoveShapePositionFacts,
    ) -> Result<(), MoveShapePositionBlock> {
        set_pos_xy_core(Some(region), &mut self.shape, x, y, facts)
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
        let _ = message
            .send_to_around(Some(&*server_region), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        server_region
            .set_move_shape_tile_position(&mut self.shape, clamped_x, clamped_y, facts)
            .map_err(MoveShapeCommandBlock::Position)?;
        Ok(true)
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
    if let Some(offset) = known_state_record_offsets(payload)
        .into_iter()
        .find(|offset| read_u32(payload, *offset) == Some(state_id))
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

// IMPLEMENTED: `CMoveShape::ForceMove` материализован выше; покрытый raw-блок
// удалён.

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
// FUNCTION: CMoveShape::RemoveState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:725
// RVA: 0x000CDAB0
// ADDRESS: 004cdab0
// PROTOTYPE: void __thiscall RemoveState(CState * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::RemoveState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:744
// RVA: 0x000CDB20
// ADDRESS: 004cdb20
// PROTOTYPE: void __thiscall RemoveState(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `AutoStartPassiveSkill` RVA `0x000CDBB0`
// материализован в owner-е выше и вызывается точным `AddObject` caller-ом.

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

// ============================================================================
// FUNCTION: CMoveShape::AddToByteArray_ForClient
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1779
// RVA: 0x000CDD30
// ADDRESS: 004cdd30
// PROTOTYPE: bool __thiscall AddToByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Реализовано выше: CShape prefix, died-byte, ordered state triples и special
// CTeamState name-tail. Неизвестный legacy record безопасно блокирует snapshot,
// потому что его недоказанный размер не позволяет вычислить следующий offset.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelUndeadState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1950
// RVA: 0x000CDE80
// ADDRESS: 004cde80
// PROTOTYPE: uint __thiscall DelUndeadState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetUndeadState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1971
// RVA: 0x000CDEF0
// ADDRESS: 004cdef0
// PROTOTYPE: uint __thiscall GetUndeadState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::StopAllSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2096
// RVA: 0x000CDF50
// ADDRESS: 004cdf50
// PROTOTYPE: void __thiscall StopAllSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::StartAllStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2234
// RVA: 0x000CE050
// ADDRESS: 004ce050
// PROTOTYPE: void __thiscall StartAllStates(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
// FUNCTION: CMoveShape::DelExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3299
// RVA: 0x000CE9C0
// ADDRESS: 004ce9c0
// PROTOTYPE: uint __thiscall DelExState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelExStateByType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3321
// RVA: 0x000CEA30
// ADDRESS: 004cea30
// PROTOTYPE: uint __thiscall DelExStateByType(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelExStateNew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3343
// RVA: 0x000CEAA0
// ADDRESS: 004ceaa0
// PROTOTYPE: uint __thiscall DelExStateNew(ulong param_1)
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
// FUNCTION: CMoveShape::DelCHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3603
// RVA: 0x000CEBD0
// ADDRESS: 004cebd0
// PROTOTYPE: uint __thiscall DelCHBYState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetCHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3625
// RVA: 0x000CEC40
// ADDRESS: 004cec40
// PROTOTYPE: uint __thiscall GetCHBYState(ulong param_1)
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
// FUNCTION: CMoveShape::ClearAllStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2132
// RVA: 0x000CF090
// ADDRESS: 004cf090
// PROTOTYPE: void __thiscall ClearAllStates(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
// FUNCTION: CMoveShape::UpdateProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:93
// RVA: 0x000CFB60
// ADDRESS: 004cfb60
// PROTOTYPE: void __thiscall UpdateProperty(void)
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
// FUNCTION: CMoveShape::UpdateAbnormality
// STATUS: PARTIALLY_MATERIALIZED_KNOWN_STATE_OWNERS
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:247
// RVA: 0x000CFD00
// ADDRESS: 004cfd00
// PROTOTYPE: void __thiscall UpdateAbnormality(void)
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
// FUNCTION: CMoveShape::ApplyFinalDamage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1607
// RVA: 0x000D0DA0
// ADDRESS: 004d0da0
// PROTOTYPE: void __thiscall ApplyFinalDamage(tagAttackInformation * param_1, vector<CMoveShape::tagDamage*,std::allocator<CMoveShape::tagDamage*>_> * param_2)
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
// FUNCTION: CMoveShape::AddState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:674
// RVA: 0x000D1560
// ADDRESS: 004d1560
// PROTOTYPE: int __thiscall AddState(tagStateID param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddUndeadState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1862
// RVA: 0x000D1780
// ADDRESS: 004d1780
// PROTOTYPE: uint __thiscall AddUndeadState(ulong param_1)
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
// FUNCTION: CMoveShape::AddExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3155
// RVA: 0x000D1E40
// ADDRESS: 004d1e40
// PROTOTYPE: uint __thiscall AddExState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddExStateNew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3228
// RVA: 0x000D20D0
// ADDRESS: 004d20d0
// PROTOTYPE: uint __thiscall AddExStateNew(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::prison_check
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3404
// RVA: 0x000D2360
// ADDRESS: 004d2360
// PROTOTYPE: void __thiscall prison_check(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddCHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3523
// RVA: 0x000D2590
// ADDRESS: 004d2590
// PROTOTYPE: uint __thiscall AddCHBYState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
