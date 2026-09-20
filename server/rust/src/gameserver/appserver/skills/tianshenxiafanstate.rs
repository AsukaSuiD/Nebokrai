//! Сохранённое состояние `CTianShenXiaFanState` (`0x335`).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/skills/tianshenxiafanstate.cpp`. `Serialize` пишет три `DWORD`:
//! ID, оставшееся время и уровень, поэтому запись занимает 12 байт. Нативный
//! `Unserialize` 0x00605C20 асимметричен: без чтения часов пишет `WORD`
//! с wire +4 в timestamp (+0x2C), а level читает с +6. Factory 0x005D8A34
//! передаёт constructor(level=0, keep=0); Unserialize keep не меняет.
//! Native input занимает 10 байт с ID; общий factory сохраняет это продвижение,
//! а отдельный tracked cache-span содержит 12 байт Serialize. Неизвестный
//! исходный tail не перечитывается с +12 и не перезаписывается расширением.
//! GetRemainedTime — constant-zero 0x00601200, поэтому нормализация без часов.
//! Сохранение непрозрачного tail не объявляется native round-trip гарантией.
//! Этот legacy defect сохранён. Типизированный owner участвует в login, пересчёте
//! свойств, строгом `CBlindState::AI`, визуалах и обратном DB-кодеке.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; RTTI-ограничения
//! формул игрока не запрещают жизненный цикл региональных держателей.
//! Exact vtable 0x0066202C: End 0x006059A0 отправляет visual и вызывает базовый End 0x005DBCE0.
//! После эффекта holder перечитывается; общий virtual UpdateProperty
//! пересчитывает свойства только при фактическом удалении точной записи.
//! Прямой End и AI используют один exact-key хвост без чтения часов.
//! StartAllStates 0x004CE050 вызывает Begin(0, self); Begin 0x00605B70
//! создаёт visual, но User остаётся NULL. Для загруженной записи базовый End
//! лишь отмечает ended; удаление из контейнера User отсутствует.

//! Restart воспроизводит только Begin(NULL, holder) (0x00605B70):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Visual принадлежит экземпляру общей арены: BeginVisualEffect(1) →
//! concrete Update(0) → базовый visual-хвост.
//! Его vtable +0x30/+0x38 ведёт на 0x00601200: пакет пишет два нуля без часов.

//! OnUpdateProperties 0x00605A10: GetSufferer → RTTI CPlayer → семь запросов
//! skill properties → элементная, физическая и защитная формулы. NULL/неигрок
//! возвращает 0, отсутствующая запись skill — 1 без изменений. Visual и часов
//! в этом callback нет; результат записывается в живой tagProperty.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
};
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape};

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::player::PlayerCombatProperties;

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

use super::skillfactory::CSkillFactory;

pub(crate) const TIAN_SHEN_XIA_FAN_STATE_ID: u32 = 0x335;
pub(crate) const TIAN_SHEN_XIA_FAN_STATE_BYTES: usize = 12;

const TARGET_DEFENSE_GAIN: u32 = 109;
const TARGET_ELEMENT_RESISTANCE_GAIN: u32 = 112;
const TARGET_ELEMENT_MODIFY_GAIN: u32 = 115;
const TARGET_MINIMUM_ATTACK_GAIN: u32 = 116;
const TARGET_MAXIMUM_ATTACK_GAIN: u32 = 117;
const PHYSICAL_AVOID_GAIN: u32 = 130;
const MAGIC_AVOID_GAIN: u32 = 131;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TianShenXiaFanState {
    started_at_ms: u32,
    keep_time_ms: u32,
    level: i32,
}

impl TianShenXiaFanState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, level: i32) -> Self {
        Self { started_at_ms, keep_time_ms, level }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != TIAN_SHEN_XIA_FAN_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        // Exact `Unserialize`: WORD в timestamp, затем DWORD level с offset + 6.
        let started_at_ms = u32::from(reader.read_u16()?);
        let level = reader.read_i32()?;
        Ok(Self::new(started_at_ms, 0, level))
    }

    pub(crate) const fn state_id(self) -> u32 { TIAN_SHEN_XIA_FAN_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }
    pub(crate) const fn client_time(self) -> u32 {
        0
    }

    pub(crate) fn encoded(self) -> [u8; TIAN_SHEN_XIA_FAN_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(TIAN_SHEN_XIA_FAN_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.state_id());
        writer.write_u32(self.client_time());
        writer.write_i32(self.level);
        bytes.try_into().expect("размер состояния сошествия фиксирован")
    }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
        skill_factory: &CSkillFactory,
    ) -> PlayerCombatProperties {
        let Some(skill) = skill_factory.query_skill_base_properties(self.state_id(), self.level)
        else {
            return properties;
        };
        let element = skill.query_property(TARGET_ELEMENT_MODIFY_GAIN);
        let minimum = skill.query_property(TARGET_MINIMUM_ATTACK_GAIN);
        let maximum = skill.query_property(TARGET_MAXIMUM_ATTACK_GAIN);
        let defense = skill.query_property(TARGET_DEFENSE_GAIN);
        let resistance = skill.query_property(TARGET_ELEMENT_RESISTANCE_GAIN);
        let physical_avoid = skill.query_property(PHYSICAL_AVOID_GAIN);
        let magic_avoid = skill.query_property(MAGIC_AVOID_GAIN);
        properties.element_modify = properties.element_modify.wrapping_add(element as i32);
        properties.minimum_attack = capped_gain(properties.minimum_attack, minimum);
        properties.maximum_attack = capped_gain(properties.maximum_attack, maximum);
        properties.defense = capped_gain(properties.defense, defense);
        properties.element_resistance = capped_gain(properties.element_resistance, resistance);
        properties.attack_avoid = properties.attack_avoid.wrapping_add(physical_avoid as u16);
        properties.element_avoid = properties.element_avoid.wrapping_add(magic_avoid as u16);
        properties
    }
}

const fn capped_gain(value: u32, gain: u32) -> u32 {
    let result = value.wrapping_add(gain);
    if result > i32::MAX as u32 { i32::MAX as u32 } else { result }
}

pub(crate) fn send_tian_shen_xia_fan_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: TianShenXiaFanState,
    begin: bool,
    _now_milliseconds: impl FnMut() -> u32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.state_id() as i32);
    if begin {
        message.add_long(state.client_time() as i32);
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn update_tian_shen_xia_fan_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    if target.object_type != 400 {
        return false;
    }
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key)).copied()
    else { return false; };
    let Some(player) = game.find_player(target.id) else { return false; };
    let properties = state.apply_to_player(player.combat_properties(), game.skill_factory());
    if let Some(player) = game.find_player_mut(target.id) {
        player.update_state_combat_properties(|_| properties);
    }
    true
}

pub(crate) fn restart_tian_shen_xia_fan_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key)).copied()
        else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.state_id() as i32);
        message.add_long(0);
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    true
}

pub(crate) fn update_tian_shen_xia_fan_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_tian_shen_xia_fan_state(game, region_id, holder, key)
}

pub(crate) fn end_tian_shen_xia_fan_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TianShenXiaFanState>(key))
        .copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.state_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    end_base_applied_state(game, region_id, holder, key, TIAN_SHEN_XIA_FAN_STATE_BYTES)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp

// ============================================================================
// FUNCTION: CTianShenXiaFanState::CTianShenXiaFanState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:19
// RVA: 0x00205780
// ADDRESS: 00605780
// PROTOTYPE: undefined __thiscall CTianShenXiaFanState(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTianShenXiaFanState::~CTianShenXiaFanState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:29
// RVA: 0x00205800
// ADDRESS: 00605800
// PROTOTYPE: void __thiscall ~CTianShenXiaFanState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTianShenXiaFanState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:79
// RVA: 0x00205810
// ADDRESS: 00605810
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTianShenXiaFanState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:97
// RVA: 0x002058D0
// ADDRESS: 006058d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTianShenXiaFanState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:141
// RVA: 0x002059C0
// ADDRESS: 006059c0
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// CTianShenXiaFanState::OnUpdateProperties (0x00605A10) реализован
// в update_tian_shen_xia_fan_state_properties; guards и порядок формул
// зафиксированы в заголовке владельца.

// ============================================================================
// FUNCTION: CTianShenXiaFanState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:64
// RVA: 0x00205B70
// ADDRESS: 00605b70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTianShenXiaFanState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:152
// RVA: 0x00205C20
// ADDRESS: 00605c20
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTianShenXiaFanStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\tianshenxiafanstate.cpp:163
// RVA: 0x00205C50
// ADDRESS: 00605c50
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
