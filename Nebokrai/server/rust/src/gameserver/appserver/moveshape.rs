//! Реализованная часть `CMoveShape` исторического GameServer.
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
//! игровое поведение читает типизированные состояния. Добавление, замена,
//! таймеры и удаление обновляют типизированную модель и её кодек в одной
//! операции с прежними смещениями и порядком.
//! Сбор душ хранится здесь без таймера и без параллельной raw-записи. Печать,
//! паутина и оглушение дополнительно сохраняют общий порядок вставки для
//! завершения через унаследованное защитное действие `CBlindState`.
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
//! Доступ к старому кодеку с порядком байтов от младшего к старшему выполняют
//! общие `LegacyReader` и `LegacyWriter`; размещение записей и их смещения
//! остаются у этого владельца.
//!
//! Реализованные `AddSkill`, `DelSkill`, `AddState`, `GetStatesNum` и
//! `UpdateAbnormality` используют это же хранилище. Ещё не восстановленные
//! классы навыков и ИИ остаются в сохранённом `UNKNOWN` (исследовательский декомпилят хранится локально) ниже.

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use indexmap::IndexSet;

use super::ai::baseai::{AiShapeAction, CBaseAI};
use super::chbystate::{CHANGE_BODY_STATE_ID, ChangeBodyMutation, ChangeBodyState};
use super::exstate::{
    EX_STATE_ID, EX_STATE_NEW_ID, ExtendedState, ExtendedStateKind, ExtendedStateMutation,
};
use super::legacycodec::{LegacyReader, LegacyWriter};
use super::region::{CRegion, RegionCellAccessBlock};
use super::ridestate::{RIDE_STATE_ID, RideState};
use super::serverregion::{CServerRegion, RegionMembershipBlock};
use super::shape::{
    CShape, SHAPE_CHANGE_AREA, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapePositionDispatch, ShapeResolver,
};
use crate::gameserver::appserver::skills::agilitystate::{
    AgilityState, PersistentAgilityFamilyState,
};
use crate::gameserver::appserver::skills::callositystate::CallosityState;
use crate::gameserver::appserver::skills::curestate::CureState;
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
use crate::gameserver::appserver::skills::roarstate::RoarState;
use crate::gameserver::appserver::skills::energyholdingstate::EnergyHoldingState;
use crate::gameserver::appserver::skills::lifeshieldstate::LifeShieldState;
use crate::gameserver::appserver::skills::machineshieldstate::MachineShieldState;
use crate::gameserver::appserver::skills::manashieldstate::ManaShieldState;
use crate::gameserver::appserver::skills::knockoutstate::KnockOutState;
use crate::gameserver::appserver::skills::boalockstate::BoaLockState;
use crate::gameserver::appserver::skills::knightcutstate::KnightCutState;
use crate::gameserver::appserver::skills::kerosenestate::{KeroseneState, KEROSENE_STATE_BYTES, KEROSENE_STATE_ID};
use crate::gameserver::appserver::skills::originstate::OriginState;
use crate::gameserver::appserver::skills::pillarstate::PillarState;
use crate::gameserver::appserver::skills::poisonarrowstate::PoisonArrowState;
use crate::gameserver::appserver::skills::poisonfogstate::{PoisonFogState, POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID};
use crate::gameserver::appserver::skills::meteorarrowstate::{MeteorArrowState, METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES};
use crate::gameserver::appserver::skills::promotionstate::PromotionState;
use crate::gameserver::appserver::skills::spiderpoisonstate::SpiderPoisonState;
use crate::gameserver::appserver::skills::spiderwebstate::SpiderWebState;
use crate::gameserver::appserver::skills::sealstate::SealState;
use crate::gameserver::appserver::skills::swordshipstate::SwordshipState;
use crate::gameserver::appserver::skills::bloodlossstate::BloodLossState;
use crate::gameserver::appserver::skills::leafcutstate::{LeafCutState, LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID};
use crate::gameserver::appserver::skills::leafcutstate3::{LeafCutState3, LEAF_CUT_3_STATE_BYTES, LEAF_CUT_3_STATE_ID};
use crate::gameserver::appserver::skills::battlefairyattributestate::BattleFairyAttributeState;
use crate::gameserver::appserver::skills::bossbluefurystate::{
    BossBlueFuryState, BossBlueFuryTick,
};
use crate::gameserver::appserver::skills::bossbluequakestate::BossBlueQuakeState;
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::appserver::skills::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::skills::taijistate::TaiJiState;
use crate::gameserver::appserver::skills::weakstate::WeakState;
use crate::gameserver::appserver::skills::godblessstate::GodBlessState;
use crate::gameserver::appserver::skills::soulcollectstate::SoulCollectState;
use crate::gameserver::appserver::states::automaticrestore::AutomaticRestoreState;
use crate::nets::netserver::message::{CMessage, GameServerAroundRuntime};
use crate::public::tools::get_line_direction;

const NPC_TYPE: i32 = 500;
const SET_POSITION_MESSAGE: i32 = 0xBF603;
const FORCE_MOVE_MESSAGE: i32 = 0xBF604;
const MOVE_MESSAGE: i32 = 0xBF605;
const SKILL_TYPE_ATTACK: u32 = 0;
const SKILL_TYPE_DEFENSE: u32 = 1;
const SKILL_TYPE_STATE: u32 = 2;
const SKILL_TYPE_SUMMON: u32 = 3;
const SKILL_NOT_DISAPPEAR_AFTER_DEAD: u32 = 56;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const UNDEAD_STATE_ID: u32 = 0x38;
const UNDEAD_STATE_PARAMETER_BYTES: usize = 72;
pub(crate) const STATE_USER_GOODS_ENLARGE_MAX_HP: i32 = 100_007;
pub(crate) const STATE_USER_GOODS_ENLARGE_MAX_MP: i32 = 100_008;
pub(crate) const STATE_IMPROVE_EXP: i32 = 100_009;
pub(crate) const STATE_USER_GOODS_ENLARGE_DEF: i32 = 100_010;
pub(crate) const STATE_USER_GOODS_ENLARGE_ELM_DEF: i32 = 100_011;
pub(crate) const STATE_USER_GOODS_ENLARGE_FULL_MISS: i32 = 100_012;
pub(crate) const STATE_AUTO_PROTECT: i32 = 110_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMoveState {
    state_id: i32,
    keep_time: u32,
    coefficient: u32,
    visual_pending: bool,
}

impl ScriptMoveState {
    pub(crate) const fn state_id(self) -> i32 {
        self.state_id
    }

    pub(crate) const fn keep_time(self) -> u32 {
        self.keep_time
    }

    pub(crate) const fn coefficient(self) -> u32 {
        self.coefficient
    }

    pub(crate) const fn is_auto_protect(self) -> bool {
        self.state_id == STATE_AUTO_PROTECT
    }
}

/// Достигнутая common-проекция `CSkill`: identity, level, category и name.
/// Исполнение concrete attack/defense/state/summon owners остаётся у самих
/// skill owners; здесь хранится точный результат `CMoveShape::AddSkill`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapeSkill {
    id: u32,
    level: i32,
    skill_type: u32,
    name: Vec<u8>,
    item_position: i32,
}

/// Достигнутый wire/lifecycle owner `CNotDisappearAfterDead`.
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
        if self.keep_time_ms == 0 {
            0
        } else {
            self.keep_time_ms
                .saturating_sub(now_ms.wrapping_sub(self.started_ms))
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

    fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }

    fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.keep_time_ms < now_ms.wrapping_sub(self.started_ms)
    }

    fn item_due(&self, now_ms: u32) -> bool {
        self.frequency_ms != 0
            && self.item_index != 0
            && self.item_amount != 0
            && self.frequency_ms < now_ms.wrapping_sub(self.last_item_tick_ms)
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
        self.skill_type
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
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

pub(crate) trait MoveShapeCommandContext {
    /// Возвращает concrete AI и соответствующий realtime timestamp одним
    /// snapshot либо `None`, если AI у shape отсутствует.
    fn ai_with_realtime_ms(&mut self, identity: ShapeIdentity) -> Option<(&mut CBaseAI, u32)>;
}

pub(crate) trait MoveShapeResolver: ShapeResolver {
    /// `Some` означает успешный RTTI `CShape -> CMoveShape`; значение хранит
    /// exact `!IsDied`, полученный у concrete derived owner-а.
    fn move_shape_is_alive(&self, identity: ShapeIdentity) -> Option<bool>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMoveShape {
    shape: CShape,
    skills: BTreeMap<u32, MoveShapeSkill>,
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
    agility_state_2: Option<AgilityState>,
    callosity_state: Option<CallosityState>,
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
    cure_state: Option<CureState>,
    daub_poison_state: Option<DaubPoisonState>,
    seal_state: Option<SealState>,
    curable_state_order: IndexSet<u32>,
    poison_arrow_state: Option<PoisonArrowState>,
    poison_fog_state: Option<PoisonFogState>,
    meteor_arrow_state: Option<MeteorArrowState>,
    spider_poison_state: Option<SpiderPoisonState>,
    spider_web_state: Option<SpiderWebState>,
    weak_state: Option<WeakState>,
    god_bless_state: Option<GodBlessState>,
    soul_collect_state: Option<SoulCollectState>,
    reached_property_state_order: u32,
    weak_state_order: Option<u32>,
    poison_fog_state_order: Option<u32>,
    god_bless_state_order: Option<u32>,
    roar_state_order: Option<u32>,
    knock_out_state: Option<KnockOutState>,
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
    leaf_cut_3_state: Option<LeafCutState3>,
    kerosene_state: Option<KeroseneState>,
    swordship_states: Vec<SwordshipState>,
    wuxing_states: Vec<super::skills::wuxingstate::WuXingState>,
    automatic_restore_states: Vec<AutomaticRestoreState>,
    battle_fairy_attribute_states: Vec<BattleFairyAttributeState>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReachedPropertyState {
    Weak(WeakState),
    PoisonFog(PoisonFogState),
    GodBless(GodBlessState),
    Roar(RoarState),
}

impl Default for CMoveShape {
    fn default() -> Self {
        Self {
            shape: CShape::default(),
            skills: BTreeMap::new(),
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
        }
    }
}

impl CMoveShape {
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

    pub(crate) fn skill_level(&self, skill_id: u32) -> i32 {
        self.skills.get(&skill_id).map_or(0, MoveShapeSkill::level)
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
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

    pub(crate) const fn skills(&self) -> &BTreeMap<u32, MoveShapeSkill> {
        &self.skills
    }

    pub(crate) fn undead_states(&self) -> &[UndeadState] {
        &self.undead_states
    }

    pub(crate) fn serialized_ex_states(&self, now_ms: u32) -> Vec<u8> {
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
        if let Some(state) = self.leaf_cut_state {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        if let Some(state) = self.leaf_cut_3_state {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        if let Some(state) = self.kerosene_state { state.update_serialized_runtime(&mut payload, now_ms); }
        if let Some(state) = self.poison_fog_state { state.update_serialized_runtime(&mut payload, now_ms); }
        if let Some(state) = self.meteor_arrow_state { state.update_serialized(&mut payload); }
        payload
    }

    pub(crate) fn replace_ex_states(&mut self, states: Vec<u8>) {
        let known_offsets = known_state_record_offsets(&states);
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
        self.periodic_attack_order.shift_remove(&LEAF_CUT_STATE_ID);
        self.leaf_cut_state = known_offsets
            .iter()
            .copied()
            .find(|offset| read_u32(&states, *offset) == Some(LEAF_CUT_STATE_ID))
            .and_then(|offset| LeafCutState::decode(&states, offset, 0).ok());
        if let Some(state) = self.leaf_cut_state {
            self.periodic_attack_order.insert(state.skill_id());
        }
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
        self.poison_fog_state_order = None;
        if let Some(state) = self.poison_fog_state { self.curable_state_order.insert(state.skill_id()); self.reached_property_state_order = self.reached_property_state_order.wrapping_add(1); self.poison_fog_state_order = Some(self.reached_property_state_order); }
        self.meteor_arrow_state = known_offsets.iter().copied()
            .find(|offset| read_u32(&states, *offset) == Some(METEOR_ARROW_MASS_SKILL_ID))
            .and_then(|offset| MeteorArrowState::decode(&states, offset).ok());
        self.ex_states.replace(states);
    }

    pub(crate) fn clear_persisted_runtime_state(&mut self) {
        self.skills.clear();
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
        self.cure_state = None;
        self.seal_state = None;
        self.curable_state_order.clear();
        self.poison_arrow_state = None;
        self.poison_fog_state = None;
        self.spider_poison_state = None;
        self.spider_web_state = None;
        self.weak_state = None;
        self.god_bless_state = None;
        self.soul_collect_state = None;
        self.reached_property_state_order = 0;
        self.weak_state_order = None;
        self.poison_fog_state_order = None;
        self.god_bless_state_order = None;
        self.roar_state_order = None;
        self.knock_out_state = None;
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
        self.leaf_cut_3_state = None;
        self.kerosene_state = None;
        self.wuxing_states.clear();
        self.automatic_restore_states.clear();
        self.battle_fairy_attribute_states.clear();
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
            || self.state_storage.cure_state.is_some()
            || self.state_storage.daub_poison_state.is_some()
            || self.state_storage.seal_state.is_some()
            || self.state_storage.poison_arrow_state.is_some()
            || self.state_storage.poison_fog_state.is_some()
            || self.state_storage.spider_poison_state.is_some()
            || self.state_storage.spider_web_state.is_some()
            || self.state_storage.weak_state.is_some()
            || self.state_storage.god_bless_state.is_some()
            || self.state_storage.soul_collect_state.is_some()
            || self.state_storage.knock_out_state.is_some()
            || self.state_storage.boa_lock_state.is_some()
            || self.state_storage.rush_state.is_some()
            || self.state_storage.rush_2_state.is_some()
            || self.state_storage.roar_state.is_some()
            || self.state_storage.pillar_state.is_some()
            || self.state_storage.knight_cut_state.is_some()
            || self.state_storage.blood_loss_state.is_some()
            || self.state_storage.kerosene_state.is_some()
            || self.state_storage.leaf_cut_state.is_some()
            || self.state_storage.leaf_cut_3_state.is_some()
            || !self.state_storage.battle_fairy_attribute_states.is_empty()
            || !self.state_storage.defense_shields.is_empty()
            || !self.state_storage.change_body_states.is_empty()
            || !self.state_storage.extended_states.is_empty()
            || !self.state_storage.undead_states.is_empty()
            || self.state_storage.ride_state.is_some()
    }

    pub(crate) fn restore_automatic_hp_mp_states(
        &mut self,
        properties: super::player::PlayerCombatProperties,
    ) {
        self.automatic_restore_states = AutomaticRestoreState::restored(properties).into();
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
    ) -> Option<ScriptMoveState> {
        let supported = matches!(
            state_id,
            STATE_USER_GOODS_ENLARGE_MAX_HP
                | STATE_USER_GOODS_ENLARGE_MAX_MP
                | STATE_IMPROVE_EXP
                | STATE_USER_GOODS_ENLARGE_DEF
                | STATE_USER_GOODS_ENLARGE_ELM_DEF
                | STATE_USER_GOODS_ENLARGE_FULL_MISS
                | STATE_AUTO_PROTECT
        );
        if !supported || (state_id == STATE_AUTO_PROTECT && sufferer_is_gm) {
            return None;
        }
        let state = ScriptMoveState {
            state_id,
            keep_time: value1 as u32,
            coefficient: if state_id == STATE_AUTO_PROTECT {
                0
            } else {
                value2 as u32
            },
            // У `AutoProtect` зацикленный визуальный эффект публикует начало
            // прямо из `Begin`; остальные эффекты ждут `UpdateProperty`.
            visual_pending: state_id != STATE_AUTO_PROTECT,
        };
        self.script_states.push(state);
        Some(state)
    }

    pub(crate) fn script_state_count(&self, state_id: i32) -> u32 {
        let scripted = self
            .script_states
            .iter()
            .filter(|state| state.state_id == state_id)
            .count();
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
        let cure = usize::from(
            self.cure_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
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
        let boa_lock = usize::from(self.boa_lock_state.is_some_and(|state| state.skill_id() as i32 == state_id));
        let rush = usize::from(
            self.rush_state
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
        let leaf_cut_3 = usize::from(
            self.leaf_cut_3_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let kerosene = usize::from(
            self.kerosene_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let battle_fairy_attributes = self
            .battle_fairy_attribute_states
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        let shields = self
            .defense_shields
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        scripted
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
            .saturating_add(spider_web)
            .saturating_add(weak)
            .saturating_add(god_bless)
            .saturating_add(soul_collect)
            .saturating_add(knock_out)
            .saturating_add(boa_lock)
            .saturating_add(rush)
            .saturating_add(pillar)
            .saturating_add(knight_cut)
            .saturating_add(blood_loss)
            .saturating_add(leaf_cut)
            .saturating_add(leaf_cut_3)
            .saturating_add(kerosene)
            .saturating_add(battle_fairy_attributes)
            .saturating_add(shields)
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
                .cure_state
                .is_some_and(|state| state.skill_id() == state_id)
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
                .battle_fairy_attribute_states
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .defense_shields
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .script_states
                .iter()
                .any(|state| state.state_id as u32 == state_id)
            || self
                .change_body_states
                .iter()
                .any(|state| state.level == state_id)
            || self
                .extended_states
                .iter()
                .any(|state| state.level == state_id)
            || self
                .undead_states
                .iter()
                .any(|state| state.state_id() == state_id)
    }

    pub(crate) fn callosity_state(&self) -> Option<CallosityState> {
        self.state_storage.callosity_state
    }

    pub(crate) fn take_callosity_state(&mut self, skill_id: u32) -> Option<CallosityState> {
        self.callosity_state
            .filter(|state| state.skill_id() == skill_id)?;
        self.callosity_state.take()
    }

    pub(crate) fn begin_callosity_state(&mut self, state: CallosityState) {
        debug_assert!(self.callosity_state.is_none());
        self.callosity_state = Some(state);
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
        state: super::skills::wuxingstate::WuXingState,
    ) -> Option<super::skills::wuxingstate::WuXingState> {
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

    pub(crate) fn wuxing_states(&self) -> &[super::skills::wuxingstate::WuXingState] {
        &self.wuxing_states
    }

    pub(crate) const fn taiji_state(&self) -> Option<TaiJiState> {
        self.state_storage.taiji_state
    }

    pub(crate) fn replace_taiji_state(&mut self, state: TaiJiState) -> Option<TaiJiState> {
        self.taiji_state.replace(state)
    }

    pub(crate) fn replace_enlarge_max_hp_state(
        &mut self,
        state: EnlargeMaxHpState,
    ) -> Option<EnlargeMaxHpState> {
        self.enlarge_max_hp_state.replace(state)
    }

    pub(crate) fn replace_enlarge_full_miss_state(
        &mut self,
        state: EnlargeFullMissState,
    ) -> Option<EnlargeFullMissState> {
        self.enlarge_full_miss_state.replace(state)
    }

    pub(crate) const fn enlarge_full_miss_state(&self) -> Option<EnlargeFullMissState> {
        self.state_storage.enlarge_full_miss_state
    }

    pub(crate) fn replace_enlarge_max_mp_state(
        &mut self,
        state: EnlargeMaxMpState,
    ) -> Option<EnlargeMaxMpState> {
        self.enlarge_max_mp_state.replace(state)
    }

    pub(crate) const fn enlarge_max_hp_state(&self) -> Option<EnlargeMaxHpState> {
        self.state_storage.enlarge_max_hp_state
    }

    pub(crate) const fn enlarge_max_mp_state(&self) -> Option<EnlargeMaxMpState> {
        self.state_storage.enlarge_max_mp_state
    }

    pub(crate) fn replace_origin_state(&mut self, state: OriginState) -> Option<OriginState> {
        self.origin_state.replace(state)
    }

    pub(crate) const fn origin_state(&self) -> Option<OriginState> {
        self.state_storage.origin_state
    }

    pub(crate) fn replace_hearten_state(&mut self, state: HeartenState) -> Option<HeartenState> {
        self.hearten_state.replace(state)
    }

    pub(crate) const fn hearten_state(&self) -> Option<HeartenState> {
        self.state_storage.hearten_state
    }

    pub(crate) fn take_expired_hearten_state(&mut self, now_ms: u32) -> Option<HeartenState> {
        self.hearten_state.filter(|state| state.expired(now_ms))?;
        self.hearten_state.take()
    }

    pub(crate) fn replace_heal_state(
        &mut self,
        removed_skill_id: u32,
        state: HealState,
    ) -> Option<HealState> {
        let previous = self
            .heal_states
            .iter()
            .position(|candidate| candidate.skill_id() == removed_skill_id)
            .map(|position| self.heal_states.remove(position));
        self.heal_states.push(state);
        previous
    }

    pub(crate) fn take_heal_states(&mut self) -> Vec<HealState> {
        std::mem::take(&mut self.heal_states)
    }

    pub(crate) fn restore_heal_states(&mut self, states: Vec<HealState>) {
        debug_assert!(self.heal_states.is_empty());
        self.heal_states = states;
    }

    pub(crate) fn push_fury_state(&mut self, state: FuryState) {
        self.fury_states.push(state);
    }

    pub(crate) fn fury_states(&self) -> &[FuryState] {
        &self.fury_states
    }

    pub(crate) fn take_expired_fury_states(&mut self, now_ms: u32) -> Vec<FuryState> {
        let mut expired = Vec::new();
        let mut position = 0;
        while position < self.fury_states.len() {
            if self.fury_states[position].expired(now_ms) {
                expired.push(self.fury_states.remove(position));
            } else {
                position += 1;
            }
        }
        expired
    }

    pub(crate) const fn rage_break_state(&self) -> Option<RageBreakState> {
        self.state_storage.rage_break_state
    }

    pub(crate) fn replace_rage_break_state(&mut self, state: RageBreakState) -> Option<RageBreakState> {
        self.rage_break_state.replace(state)
    }

    pub(crate) fn take_rage_break_state(&mut self) -> Option<RageBreakState> {
        self.rage_break_state.take()
    }

    pub(crate) fn take_expired_rage_break_state(&mut self, now_ms: u32) -> Option<RageBreakState> {
        self.rage_break_state.filter(|state| state.expired(now_ms))?;
        self.rage_break_state.take()
    }

    pub(crate) fn take_boss_blue_fury_state(&mut self) -> Option<BossBlueFuryState> {
        self.boss_blue_fury_state.take()
    }

    pub(crate) fn begin_boss_blue_fury_state(&mut self, state: BossBlueFuryState) {
        debug_assert!(self.boss_blue_fury_state.is_none());
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
        }
        Some((snapshot, tick))
    }

    pub(crate) fn replace_boss_blue_quake_state(
        &mut self,
        state: BossBlueQuakeState,
    ) -> Option<BossBlueQuakeState> {
        self.curable_state_order.insert(state.skill_id());
        self.boss_blue_quake_state.replace(state)
    }

    pub(crate) fn take_expired_boss_blue_quake_state(
        &mut self,
        now_ms: u32,
    ) -> Option<BossBlueQuakeState> {
        self.boss_blue_quake_state.filter(|state| state.expired(now_ms))?;
        let state = self.boss_blue_quake_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn take_boss_blue_quake_state(&mut self) -> Option<BossBlueQuakeState> {
        let state = self.boss_blue_quake_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
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
        self.defense_shields.push(DefenseShieldState::Life(state));
        previous
    }

    /// Повторное наложение `Promotion` завершает прежнее состояние и не
    /// создаёт новое. Это подтверждённая особенность исходного `AI`.
    pub(crate) fn begin_promotion_state(&mut self, state: PromotionState) -> bool {
        if let Some(position) = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
        {
            self.defense_shields.remove(position);
            return false;
        }
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
            DefenseShieldState::Promotion(state) => Some(state),
            _ => unreachable!("позиция состояния Promotion проверена"),
        }
    }

    pub(crate) fn take_expired_defense_shields(
        &mut self,
        now_ms: u32,
        player_mana: u32,
        player_dead: bool,
        war_soul_mana: Option<i32>,
    ) -> Vec<DefenseShieldState> {
        let mut expired = Vec::new();
        let mut position = 0;
        while position < self.defense_shields.len() {
            if self.defense_shields[position].expired(
                now_ms,
                player_mana,
                player_dead,
                war_soul_mana,
            ) {
                expired.push(self.defense_shields.remove(position));
            } else {
                position += 1;
            }
        }
        expired
    }

    pub(crate) fn take_defense_shields(&mut self) -> Vec<DefenseShieldState> {
        std::mem::take(&mut self.defense_shields)
    }

    pub(crate) fn restore_defense_shields(&mut self, states: Vec<DefenseShieldState>) {
        self.defense_shields = states;
    }

    pub(crate) fn replace_cure_state(&mut self, state: CureState) -> Option<CureState> {
        self.cure_state.replace(state)
    }

    pub(crate) fn take_cure_state_for_ai(&mut self) -> Option<CureState> {
        self.cure_state.take()
    }

    pub(crate) fn replace_daub_poison_state(
        &mut self,
        state: DaubPoisonState,
    ) -> Option<DaubPoisonState> {
        self.daub_poison_state.replace(state)
    }

    pub(crate) fn take_expired_daub_poison_state(
        &mut self,
        now_ms: u32,
    ) -> Option<DaubPoisonState> {
        self.daub_poison_state
            .filter(|state| state.expired(now_ms))?;
        self.daub_poison_state.take()
    }

    pub(crate) fn replace_seal_state(&mut self, state: SealState) -> Option<SealState> {
        self.blind_state_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.seal_state.replace(state)
    }

    pub(crate) fn take_expired_seal_state(&mut self, now_ms: u32) -> Option<SealState> {
        let state = self.seal_state.filter(|state| state.expired(now_ms))?;
        self.seal_state = None;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn take_seal_state(&mut self) -> Option<SealState> {
        let state = self.seal_state.take()?;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn replace_poison_arrow_state(
        &mut self,
        state: PoisonArrowState,
    ) -> Option<PoisonArrowState> {
        self.periodic_attack_order.insert(state.skill_id());
        self.poison_arrow_state.replace(state)
    }

    pub(crate) fn take_poison_arrow_state_for_ai(&mut self) -> Option<PoisonArrowState> {
        self.poison_arrow_state.take()
    }

    pub(crate) fn replace_spider_poison_state(
        &mut self,
        state: SpiderPoisonState,
    ) -> Option<SpiderPoisonState> {
        self.periodic_attack_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.spider_poison_state.replace(state)
    }

    pub(crate) fn take_spider_poison_state_for_ai(&mut self) -> Option<SpiderPoisonState> {
        self.spider_poison_state.take()
    }

    pub(crate) fn take_spider_poison_state(&mut self) -> Option<SpiderPoisonState> {
        let state = self.spider_poison_state.take()?;
        self.periodic_attack_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn replace_spider_web_state(
        &mut self,
        state: SpiderWebState,
    ) -> Option<SpiderWebState> {
        self.blind_state_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.spider_web_state.replace(state)
    }

    pub(crate) const fn weak_state(&self) -> Option<WeakState> {
        self.state_storage.weak_state
    }

    pub(crate) fn replace_weak_state(&mut self, state: WeakState) -> Option<WeakState> {
        self.reached_property_state_order = self.reached_property_state_order.wrapping_add(1);
        self.weak_state_order = Some(self.reached_property_state_order);
        self.weak_state.replace(state)
    }

    pub(crate) fn take_weak_state(&mut self) -> Option<WeakState> {
        let state = self.weak_state.take()?;
        self.weak_state_order = None;
        Some(state)
    }

    pub(crate) fn take_weak_state_outside(&mut self, tile_x: i32, tile_y: i32) -> Option<WeakState> {
        let state = self.weak_state.filter(|state| !state.contains(tile_x, tile_y))?;
        self.weak_state = None;
        self.weak_state_order = None;
        Some(state)
    }

    pub(crate) const fn god_bless_state(&self) -> Option<GodBlessState> { self.state_storage.god_bless_state }
    pub(crate) fn replace_god_bless_state(&mut self, state: GodBlessState) -> Option<GodBlessState> {
        self.reached_property_state_order = self.reached_property_state_order.wrapping_add(1);
        self.god_bless_state_order = Some(self.reached_property_state_order);
        self.god_bless_state.replace(state)
    }
    pub(crate) const fn roar_state(&self) -> Option<RoarState> { self.state_storage.roar_state }
    pub(crate) fn replace_roar_state(&mut self, state: RoarState) -> Option<RoarState> {
        self.reached_property_state_order = self.reached_property_state_order.wrapping_add(1);
        self.roar_state_order = Some(self.reached_property_state_order);
        self.roar_state.replace(state)
    }
    pub(crate) fn take_expired_roar_state(&mut self, now_ms: u32) -> Option<RoarState> {
        let state = self.roar_state.filter(|state| state.expired(now_ms))?;
        self.roar_state = None;
        self.roar_state_order = None;
        Some(state)
    }
    pub(crate) const fn energy_holding_state(&self) -> Option<EnergyHoldingState> { self.state_storage.energy_holding_state }
    pub(crate) fn energy_holding_state_mut(&mut self) -> Option<&mut EnergyHoldingState> { self.state_storage.energy_holding_state.as_mut() }
    pub(crate) fn begin_energy_holding_state(&mut self, state: EnergyHoldingState) { self.state_storage.energy_holding_state = Some(state); }
    pub(crate) fn take_energy_holding_state(&mut self) -> Option<EnergyHoldingState> { self.state_storage.energy_holding_state.take() }
    pub(crate) fn reached_property_states(&self) -> Vec<ReachedPropertyState> {
        let current = self.reached_property_state_order;
        let mut states = Vec::with_capacity(4);
        if let (Some(order), Some(state)) = (self.weak_state_order, self.weak_state) {
            states.push((current.wrapping_sub(order), ReachedPropertyState::Weak(state)));
        }
        if let (Some(order), Some(state)) = (self.poison_fog_state_order, self.poison_fog_state) { states.push((current.wrapping_sub(order), ReachedPropertyState::PoisonFog(state))); }
        if let (Some(order), Some(state)) = (self.god_bless_state_order, self.god_bless_state) {
            states.push((current.wrapping_sub(order), ReachedPropertyState::GodBless(state)));
        }
        if let (Some(order), Some(state)) = (self.roar_state_order, self.roar_state) {
            states.push((current.wrapping_sub(order), ReachedPropertyState::Roar(state)));
        }
        states.sort_by(|left, right| right.0.cmp(&left.0));
        states.into_iter().map(|(_, state)| state).collect()
    }
    pub(crate) fn take_expired_god_bless_state(&mut self, now_ms: u32) -> Option<GodBlessState> {
        let state = self.god_bless_state.filter(|state| state.expired(now_ms))?;
        self.god_bless_state = None;
        self.god_bless_state_order = None;
        Some(state)
    }
    pub(crate) const fn soul_collect_state(&self) -> Option<SoulCollectState> {
        self.state_storage.soul_collect_state
    }

    pub(crate) fn begin_soul_collect_state(&mut self, state: SoulCollectState) {
        debug_assert!(self.soul_collect_state.is_none());
        self.soul_collect_state = Some(state);
    }

    pub(crate) fn soul_collect_state_mut(&mut self) -> Option<&mut SoulCollectState> {
        self.soul_collect_state.as_mut()
    }

    pub(crate) fn take_soul_collect_state(&mut self) -> Option<SoulCollectState> {
        self.soul_collect_state.take()
    }

    pub(crate) fn take_expired_spider_web_state(
        &mut self,
        now_ms: u32,
    ) -> Option<SpiderWebState> {
        let state = self.spider_web_state.filter(|state| state.expired(now_ms))?;
        self.spider_web_state = None;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn take_spider_web_state(&mut self) -> Option<SpiderWebState> {
        let state = self.spider_web_state.take()?;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn replace_knock_out_state(&mut self, state: KnockOutState) -> Option<KnockOutState> {
        self.blind_state_order.insert(state.skill_id());
        self.curable_state_order.insert(state.skill_id());
        self.knock_out_state.replace(state)
    }

    pub(crate) fn replace_boa_lock_state(&mut self, state: BoaLockState) -> Option<BoaLockState> { self.boa_lock_state.replace(state) }
    pub(crate) fn take_expired_boa_lock_state(&mut self, now_ms: u32) -> Option<BoaLockState> { self.boa_lock_state.filter(|state| state.expired(now_ms))?; self.boa_lock_state.take() }

    pub(crate) fn pillar_state(&self) -> Option<PillarState> { self.pillar_state }

    pub(crate) fn replace_rush_state(&mut self, state: RushState) -> Option<RushState> {
        self.rush_state.replace(state)
    }

    pub(crate) fn take_expired_rush_state(&mut self, now_ms: u32) -> Option<RushState> {
        self.rush_state.filter(|state| state.expired(now_ms))?;
        self.rush_state.take()
    }

    pub(crate) fn replace_rush_2_state(&mut self, state: Rush2State) -> Option<Rush2State> {
        self.rush_2_state.replace(state)
    }

    pub(crate) fn take_expired_rush_2_state(&mut self, now_ms: u32) -> Option<Rush2State> {
        self.rush_2_state.filter(|state| state.expired(now_ms))?;
        self.rush_2_state.take()
    }

    pub(crate) fn replace_pillar_state(&mut self, state: PillarState) -> Option<PillarState> {
        self.pillar_state.replace(state)
    }
    pub(crate) fn take_expired_pillar_state(&mut self, now_ms: u32) -> Option<PillarState> {
        self.pillar_state.filter(|state| state.expired(now_ms))?;
        self.pillar_state.take()
    }
    pub(crate) fn take_pillar_state(&mut self) -> Option<PillarState> { self.pillar_state.take() }

    pub(crate) fn take_expired_knock_out_state(&mut self, now_ms: u32) -> Option<KnockOutState> {
        let state = self.knock_out_state.filter(|state| state.expired(now_ms))?;
        self.knock_out_state = None;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn take_knock_out_state(&mut self) -> Option<KnockOutState> {
        let state = self.knock_out_state.take()?;
        self.blind_state_order.shift_remove(&state.skill_id());
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn replace_knight_cut_state(&mut self, state: KnightCutState) -> Option<KnightCutState> {
        self.curable_state_order.insert(state.skill_id());
        self.knight_cut_state.replace(state)
    }

    pub(crate) fn take_expired_knight_cut_state(&mut self, now_ms: u32) -> Option<KnightCutState> {
        let state = self.knight_cut_state.filter(|state| state.expired(now_ms))?;
        self.knight_cut_state = None;
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn take_knight_cut_state(&mut self) -> Option<KnightCutState> {
        let state = self.knight_cut_state.take()?;
        self.curable_state_order.shift_remove(&state.skill_id());
        Some(state)
    }

    pub(crate) fn curable_state_ids(&self) -> Vec<u32> {
        self.curable_state_order.iter().copied().collect()
    }

    pub(crate) fn replace_poison_fog_state(&mut self, mut state: PoisonFogState, now_ms: u32) -> Option<PoisonFogState> {
        let previous = self.poison_fog_state.take();
        let replaced = previous.and_then(PoisonFogState::serialized_span).is_some_and(|(offset, amount)| amount == POISON_FOG_STATE_BYTES && state.write_serialized_at(&mut self.ex_states, offset, now_ms));
        if !replaced { if self.ex_states.len() < 4 { self.ex_states.clear(); LegacyWriter::new(&mut self.ex_states).write_u32(0); } let count = read_u32(&self.ex_states, 0).expect("счётчик состояний"); write_u32(&mut self.ex_states, 0, count.wrapping_add(1)); state.append_serialized(&mut self.ex_states, now_ms); }
        self.curable_state_order.shift_remove(&state.skill_id()); self.curable_state_order.insert(state.skill_id()); self.reached_property_state_order = self.reached_property_state_order.wrapping_add(1); self.poison_fog_state_order = Some(self.reached_property_state_order); self.poison_fog_state = Some(state); previous
    }
    pub(crate) fn take_expired_poison_fog_state(&mut self, now_ms: u32) -> Option<PoisonFogState> { let state = self.poison_fog_state.filter(|state| state.expired(now_ms))?; self.finish_poison_fog_state(state); Some(state) }
    pub(crate) fn take_poison_fog_state(&mut self) -> Option<PoisonFogState> { let state = self.poison_fog_state?; self.finish_poison_fog_state(state); Some(state) }
    fn finish_poison_fog_state(&mut self, state: PoisonFogState) { self.poison_fog_state = None; self.poison_fog_state_order = None; self.curable_state_order.shift_remove(&state.skill_id()); let Some((offset, amount)) = state.serialized_span() else { return }; if offset.saturating_add(amount) > self.ex_states.len() { return } self.ex_states.drain(offset..offset + amount); if self.ex_states.len() >= 4 { let count = read_u32(&self.ex_states, 0).expect("счётчик состояний"); write_u32(&mut self.ex_states, 0, count.saturating_sub(1)); } for state in &mut self.extended_states { state.shift_serialized_offset_after(offset, amount); } for state in &mut self.change_body_states { state.shift_serialized_offset_after(offset, amount); } for state in &mut self.undead_states { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.leaf_cut_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.leaf_cut_3_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.kerosene_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.meteor_arrow_state { state.shift_serialized_offset_after(offset, amount); } if let Some(state) = &mut self.ride_state { state.shift_serialized_offset_after(offset, amount); } }
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
        self.blood_loss_state.replace(state)
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
                expired.push(self.battle_fairy_attribute_states.remove(position));
            } else {
                position += 1;
            }
        }
        expired
    }

    pub(crate) fn take_blood_loss_state_for_ai(&mut self) -> Option<BloodLossState> {
        self.blood_loss_state.take()
    }

    pub(crate) fn periodic_attack_state_ids(&self) -> Vec<u32> {
        self.periodic_attack_order.iter().copied().collect()
    }

    pub(crate) fn finish_periodic_attack_state(&mut self, skill_id: u32) {
        self.periodic_attack_order.shift_remove(&skill_id);
        self.curable_state_order.shift_remove(&skill_id);
    }

    pub(crate) fn agility_state(&self, skill_id: u32) -> Option<AgilityState> {
        if skill_id == crate::gameserver::appserver::skills::agility::AGILITY_2_SKILL_ID {
            return self.agility_state_2;
        }
        match self.persistent_agility_family_state {
            Some(PersistentAgilityFamilyState::Agility(state)) if state.skill_id() == skill_id => {
                Some(state)
            }
            _ => None,
        }
    }

    pub(crate) fn take_agility_state(&mut self, skill_id: u32) -> Option<AgilityState> {
        if skill_id == crate::gameserver::appserver::skills::agility::AGILITY_2_SKILL_ID {
            return self.agility_state_2.take();
        }
        match self.persistent_agility_family_state {
            Some(PersistentAgilityFamilyState::Agility(state)) if state.skill_id() == skill_id => {
                self.persistent_agility_family_state.take();
                Some(state)
            }
            _ => None,
        }
    }

    pub(crate) fn begin_agility_state(&mut self, state: AgilityState) {
        match state.skill_id() {
            crate::gameserver::appserver::skills::agility::AGILITY_SKILL_ID => {
                debug_assert!(self.persistent_agility_family_state.is_none());
                self.persistent_agility_family_state =
                    Some(PersistentAgilityFamilyState::Agility(state));
            }
            crate::gameserver::appserver::skills::agility::AGILITY_2_SKILL_ID => {
                debug_assert!(self.agility_state_2.is_none());
                self.agility_state_2 = Some(state);
            }
            _ => unreachable!("каноническое состояние ловкости имеет известный ID"),
        }
    }

    pub(crate) fn persistent_agility_family_state(
        &self,
    ) -> Option<PersistentAgilityFamilyState> {
        self.persistent_agility_family_state
    }

    pub(crate) fn take_persistent_agility_family_state(
        &mut self,
    ) -> Option<PersistentAgilityFamilyState> {
        self.persistent_agility_family_state.take()
    }

    pub(crate) fn begin_persistent_agility_family_state(
        &mut self,
        state: PersistentAgilityFamilyState,
    ) {
        debug_assert!(PersistentAgilityFamilyState::is_known_skill(
            state.skill_id()
        ));
        debug_assert!(self.persistent_agility_family_state.is_none());
        self.persistent_agility_family_state = Some(state);
    }

    pub(crate) fn take_expired_agility_state_2(&mut self, now_ms: u32) -> Option<AgilityState> {
        self.agility_state_2.filter(|state| state.expired(now_ms))?;
        self.agility_state_2.take()
    }

    pub(crate) fn take_first_script_state(&mut self, state_id: i32) -> Option<ScriptMoveState> {
        let index = self
            .script_states
            .iter()
            .position(|state| state.state_id == state_id)?;
        Some(self.script_states.remove(index))
    }

    pub(crate) fn script_states(&self) -> &[ScriptMoveState] {
        &self.script_states
    }

    pub(crate) fn take_pending_script_state_visuals(&mut self) -> Vec<ScriptMoveState> {
        let mut pending = Vec::new();
        for state in &mut self.script_states {
            if state.visual_pending {
                state.visual_pending = false;
                pending.push(*state);
            }
        }
        pending
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

    pub(crate) fn skill(&self, skill_id: u32) -> Option<&MoveShapeSkill> {
        self.skills.get(&skill_id)
    }

    pub(crate) fn set_item_skill_position(&mut self, skill_id: u32, position: i32) -> bool {
        let Some(skill) = self.skills.get_mut(&skill_id) else { return false };
        skill.set_item_position(position);
        true
    }

    /// Достигнутый ID-view `GetCurrentSkill`: concrete `CSkill` execution и
    /// его `End` остаются у ещё не перенесённого skill owner-а, но caller-ы
    /// могут точно отличить запретный active skill `0xD4`.
    pub(crate) const fn current_skill_id(&self) -> Option<u32> {
        self.current_skill_id
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

    /// Exact `AddSkill(tagSkillID, long)` для already decoded factory registry:
    /// прежний ненулевой уровень не понижается; иначе entry заменяется только
    /// для одной из четырёх canonical категорий.
    pub(crate) fn add_skill(&mut self, skill_id: u32, level: i32, factory: &CSkillFactory) -> bool {
        if let Some(existing) = self.skills.get(&skill_id) {
            if existing.level != 0 && level <= existing.level {
                return true;
            }
        }
        self.skills.remove(&skill_id);
        let Some(properties) = factory.query_skill_base_properties(skill_id, level) else {
            return false;
        };
        let skill_type = properties.skill_type();
        if !matches!(
            skill_type,
            SKILL_TYPE_ATTACK | SKILL_TYPE_DEFENSE | SKILL_TYPE_STATE | SKILL_TYPE_SUMMON
        ) {
            return false;
        }
        self.skills.insert(
            skill_id,
            MoveShapeSkill {
                id: skill_id,
                level,
                skill_type,
                name: properties.skill_name().to_vec(),
                item_position: -1,
            },
        );
        true
    }

    /// Exact reached state-transition `DelSkill(tagSkillID)`. Исходник всегда
    /// завершает текущий skill до category lookup, даже когда удаляется другой
    /// ID или искомой записи нет. Concrete `End/Delete` не имеют отдельного
    /// наблюдаемого state в достигнутой common-проекции.
    pub(crate) fn delete_skill(&mut self, skill_id: u32, factory: &CSkillFactory) -> bool {
        if skill_id == 0 {
            return false;
        }
        if self
            .current_skill_id
            .is_some_and(|current| self.skills.contains_key(&current))
        {
            self.current_skill_id = None;
        }
        if !matches!(
            factory.query_skill_type(skill_id, 1),
            SKILL_TYPE_ATTACK | SKILL_TYPE_DEFENSE | SKILL_TYPE_STATE | SKILL_TYPE_SUMMON
        ) {
            return false;
        }
        self.skills.remove(&skill_id);
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

    #[allow(
        clippy::too_many_arguments,
        reason = "literal ForceMove сохраняет исходные аргументы и две достигнутые owner-границы"
    )]
    pub(crate) fn force_move<Context: MoveShapeCommandContext>(
        &mut self,
        server_region: Option<&mut CServerRegion>,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        facts: MoveShapePositionFacts,
        around: &GameServerAroundRuntime<'_>,
        context: &mut Context,
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
        if let Some((ai, now_ms)) = context.ai_with_realtime_ms(identity) {
            ai.add_ai_event(AiShapeAction::Stand, duration_ms, 0, now_ms);
        }
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

/// Возвращает только подтверждённые начала известных записей. Размер
/// неизвестного класса из wire не выводится, поэтому после него типизация
/// прекращается, а исходный хвост остаётся в `LegacyStateCodec` без изменений.
fn known_state_record_offsets(payload: &[u8]) -> Vec<usize> {
    let Some(declared_count) = read_u32(payload, 0) else {
        return Vec::new();
    };
    let mut offsets = Vec::new();
    let mut cursor = 4usize;
    for _ in 0..declared_count {
        let Some(state_id) = read_u32(payload, cursor) else {
            break;
        };
        let size = match state_id {
            CHANGE_BODY_STATE_ID => 124,
            EX_STATE_ID => 44,
            EX_STATE_NEW_ID => 56,
            UNDEAD_STATE_ID => 76,
            LEAF_CUT_STATE_ID => LEAF_CUT_STATE_BYTES,
            LEAF_CUT_3_STATE_ID => LEAF_CUT_3_STATE_BYTES,
            KEROSENE_STATE_ID => KEROSENE_STATE_BYTES,
            POISON_FOG_STATE_ID => POISON_FOG_STATE_BYTES,
            METEOR_ARROW_MASS_SKILL_ID => METEOR_ARROW_STATE_BYTES,
            RIDE_STATE_ID => {
                let name_start = cursor.saturating_add(16);
                let Some(name) = payload.get(name_start..) else {
                    break;
                };
                let Some(length) = name.iter().take(256).position(|byte| *byte == 0) else {
                    break;
                };
                16 + length + 1
            }
            _ => break,
        };
        let Some(end) = cursor.checked_add(size).filter(|end| *end <= payload.len()) else {
            break;
        };
        offsets.push(cursor);
        cursor = end;
    }
    offsets
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
// FUNCTION: CMoveShape::Stiffen
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2316
// RVA: 0x000CD2F0
// ADDRESS: 004cd2f0
// PROTOTYPE: ulong __thiscall Stiffen(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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

// ============================================================================
// FUNCTION: CMoveShape::AutoStartPassiveSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1687
// RVA: 0x000CDBB0
// ADDRESS: 004cdbb0
// PROTOTYPE: void __thiscall AutoStartPassiveSkill(void)
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

// ============================================================================
// FUNCTION: CMoveShape::AddToByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1779
// RVA: 0x000CDD30
// ADDRESS: 004cdd30
// PROTOTYPE: bool __thiscall AddToByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
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
// FUNCTION: CMoveShape::GetDefaultAttackSkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2464
// RVA: 0x000CE240
// ADDRESS: 004ce240
// PROTOTYPE: tagSkillID __thiscall GetDefaultAttackSkillID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2677
// RVA: 0x000CE2D0
// ADDRESS: 004ce2d0
// PROTOTYPE: CSkill * __thiscall GetSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
// FUNCTION: CMoveShape::IncreaseExperienceForAllFallowers
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2854
// RVA: 0x000CE5A0
// ADDRESS: 004ce5a0
// PROTOTYPE: void __thiscall IncreaseExperienceForAllFallowers(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetCurrentPetAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2891
// RVA: 0x000CE6B0
// ADDRESS: 004ce6b0
// PROTOTYPE: void __thiscall SetCurrentPetAction(PET_ACTION param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetTargetForAllPets
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2911
// RVA: 0x000CE790
// ADDRESS: 004ce790
// PROTOTYPE: void __thiscall SetTargetForAllPets(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetCurrentPetsMode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2952
// RVA: 0x000CE8C0
// ADDRESS: 004ce8c0
// PROTOTYPE: void __thiscall SetCurrentPetsMode(PET_SEARCH_ENEMY_MODE param_1)
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
// FUNCTION: CMoveShape::ClearSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:192
// RVA: 0x000CECE0
// ADDRESS: 004cece0
// PROTOTYPE: void __thiscall ClearSkills(void)
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
