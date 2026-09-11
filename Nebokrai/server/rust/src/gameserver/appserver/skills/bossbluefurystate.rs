//! Каноническое состояние ярости синего босса `CBossBlueFuryState` (`0x1f7`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluefurystate.cpp`. Достигнутые пути игрока и монстра
//! заменяют первый найденный экземпляр до установки нового, сохраняя остальные
//! загруженные записи. Движение и бой запрещены на слабой фазе, после неё
//! состояние сохраняется до общего срока. Только для монстра минимальная
//! и максимальная атака заменяются указанной долей коэффициента: полный unsigned
//! attack и signed-коэффициент перемножаются в x87 с `0.01_f32`, затем результат
//! усекается в `i32`. Визуальные начало и завершение сохраняют `0xBFE03/04`.
//! DB-запись содержит остаток срока и коэффициент атаки;
//! `weak_time` после загрузки остаётся нулевым.
//! AI получает один ключ общей арены; общий CMoveShape задаёт порядок прохода.
//! Exact AI `0x005E8D50` отдельно читает часы перед слабой и общей границами.
//! После слабой границы запреты снимаются на каждом AI, без one-shot флага.
//! End `0x005E8D10` отправляет эффект, удаляет тот же экземпляр и отдельно
//! снимает оба запрета; окончание срока не подменяет проверку слабой границы.
//! Прямой End и AI используют один exact-key хвост; только AI читает часы.

//! Restart воспроизводит только Begin(NULL, holder) (0x005E8ED0):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Begin создаёт принадлежащий записи loop=1 visual без немедленного пакета.
//! Оба запрета добавляются до выделения visual, включая loaded weak_time=0.

//! Unserialize 0x005D6190 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual,
};
use crate::gameserver::appserver::moveshape::StateKey;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp

// ============================================================================
// FUNCTION: CBossBlueFuryState::CBossBlueFuryState
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная ветвь монстра выполняется `BossBlueFuryState::new`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:15
// RVA: 0x001E8A60
// ADDRESS: 005e8a60
// PROTOTYPE: undefined __thiscall CBossBlueFuryState(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::CBossBlueFuryState
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная ветвь монстра выполняется `BossBlueFuryState::new`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:26
// RVA: 0x001E8AE0
// ADDRESS: 005e8ae0
// PROTOTYPE: undefined __thiscall CBossBlueFuryState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::~CBossBlueFuryState
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение достигнутых путей выполняет `end_boss_blue_fury_state`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:37
// RVA: 0x001E8B50
// ADDRESS: 005e8b50
// PROTOTYPE: void __thiscall ~CBossBlueFuryState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:89
// RVA: 0x001E8B60
// ADDRESS: 005e8b60
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:106
// RVA: 0x001E8C30
// ADDRESS: 005e8c30
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::End
// STATUS: IMPLEMENTED
// Завершение достигнутых путей выполняют функции истечения для игрока и монстра,
// а также ветви замены в соответствующих исполнителях навыка.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:123
// RVA: 0x001E8D10
// ADDRESS: 005e8d10
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::AI
// STATUS: IMPLEMENTED
// Слабая и общая границы читают часы раздельно в concrete single-key AI.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:147
// RVA: 0x001E8D50
// ADDRESS: 005e8d50
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::OnUpdateProperties
// STATUS: PARTIALLY_IMPLEMENTED
// Формула монстра выполняется `BossBlueFuryState::apply_to_monster_attack`;
// исходная проверка типа `600` не применяет коэффициент к игроку.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:48
// RVA: 0x001E8DC0
// ADDRESS: 005e8dc0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектные ветви игрока и монстра выполняются `BossBlueFuryState::new`
// и каноническим владельцем в `CMoveShape`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:75
// RVA: 0x001E8ED0
// ADDRESS: 005e8ed0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryStateVisualEffect::UpdateVisualEffect
// STATUS: IMPLEMENTED
// Достигнутый путь выполняет `send_boss_blue_fury_state_visual`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:204
// RVA: 0x001E8F80
// ADDRESS: 005e8f80
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Restart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:142
// RVA: 0x001FD450
// ADDRESS: 005fd450
// PROTOTYPE: void __thiscall Restart(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BOSS_BLUE_FURY_STATE_ID: u32 = 0x1f7;
pub(crate) const BOSS_BLUE_FURY_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossBlueFuryState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_factor_percent: i32,
    weak_time_ms: u32,
}



impl BossBlueFuryState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        attack_factor_percent: i32,
        weak_time_ms: u32,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            attack_factor_percent,
            weak_time_ms,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        BOSS_BLUE_FURY_STATE_ID
    }



    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BOSS_BLUE_FURY_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?, 0))
    }

    pub(crate) fn encoded(self, now_ms: u32) -> [u8; BOSS_BLUE_FURY_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(BOSS_BLUE_FURY_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(BOSS_BLUE_FURY_STATE_ID);
        writer.write_u32(self.client_time(|| now_ms) as u32);
        writer.write_i32(self.attack_factor_percent);
        bytes.try_into().expect("размер состояния ярости синего босса фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; BOSS_BLUE_FURY_STATE_BYTES] {
        self.encoded(self.started_at_ms)
    }



    pub(crate) const fn weak_elapsed(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.weak_time_ms) < now_ms
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn apply_to_monster_attack(self, attack: u32) -> u32 {
        truncate_original(
            f64::from(self.attack_factor_percent)
                * f64::from(0.01_f32)
                * f64::from(attack),
        ) as u32
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_boss_blue_fury_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BossBlueFuryState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(|| now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn restart_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).is_none() {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(false);
        shape.set_fightable(false);
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    mut now_milliseconds: impl FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).copied()
        else { return false };
    if state.weak_elapsed(now_milliseconds()) {
        if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
            shape.set_moveable(true);
            shape.set_fightable(true);
        }
    }
    if state.expired(now_milliseconds()) {
        let _ = end_boss_blue_fury_state(game, region_id, holder, key);
    }
    true
}

pub(crate) fn end_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<BossBlueFuryState>(
            key, BOSS_BLUE_FURY_STATE_BYTES,
        )).is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(true);
        shape.set_fightable(true);
    }
    removed
}

pub(crate) fn end_player_boss_blue_fury_state_key(
    game: &mut CGame,
    player_id: i32,
    key: StateKey,
    _now_ms: u32,
) -> bool {
    let Some((region_id, holder)) = game.find_player(player_id)
        .and_then(|player| Some((player.server_region_id()?, player.shape().identity())))
        else { return false };
    end_boss_blue_fury_state(game, region_id, holder, key)
}
