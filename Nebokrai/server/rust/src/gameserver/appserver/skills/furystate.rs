//! Каноническое накапливаемое состояние `CFuryState` (`0x1a3`).
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает добавление без
//! замены, строгую проверку истечения и последовательное
//! процентное увеличение только максимальной атаки игрока или монстра.
//! Произведение signed-прибавки, `0.01_f32` и полного unsigned-максимума
//! вычисляется в x87, после чего `__ftol2` усекает его к нулю; игрок использует
//! младшие 16 бит результата. Визуальные начало/завершение сохраняют
//! `0xBFE03/04`.
//! `GetRemainedTime` разделяет exact-тело `CFuryState` по `0x00605E10`:
//! положительный остаток вычисляется после отдельного второго чтения часов.
//! Serializer `0x005E7330` и exact `Unserialize` `0x005FD660` задают
//! 12-байтовую запись `ID + remaining time + attack gain`; несколько записей
//! сохраняют исходный порядок наложения. Общие property/End разрешают
//! опубликованного регионального владельца.
//! Vtable `0x0065FB6C` связывает AI (`0x005EA4C0`) со строгим сроком,
//! а End (`0x006059A0` → `0x005DBCE0`) — с эффектом перед RemoveState.
//! Каждый экземпляр удаляется отдельно с собственным пересчётом держателя;
//! захваченный поколенческий ключ не подменяется после внешнего эффекта.
//! Порядок живых ключей сохраняет соответствие повторных DB-записей,
//! удаление оставляет пустой слот до общего уплотнения CMoveShape.
//! Общий CMoveShape::UpdateAbnormality передаёт один ключ; этот owner
//! не запускает отдельный семейный обход и сохраняет границы своих callbacks.
//! Прямой End и AI используют один exact-key хвост без чтения часов.
//! StartAllStates 0x004CE050 вызывает Begin(0, self); Begin 0x005EA500
//! сразу возвращает при NULL User без visual. Загруженный экземпляр поэтому
//! только получает отметку ended; для runtime self-cast User совпадает с holder
//! и базовый End удаляет точную запись.

//! Begin(NULL, holder) (0x005EA500) возвращает 0 на проверке User до базы:
//! restart не меняет время, ended или прежний visual-ресурс.
//! При непустом User точный Begin0x005EA534–0x005EA596 выполняет базовый
//! Begin, выделение visual и BeginVisualEffect(1), затем return1 без Update.
//! Поэтому runtime-наложение не отправляет BFE03 до общего property-прохода.

//! Unserialize 0x005FD660 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

//! Общий с RageBreak OnUpdateProperties 0x005FD480: GetSufferer → существующий
//! visual Update(0) → RTTI-формула. Игрок повторно сужает прибавку до WORD
//! после ограничения суммы 0xFFFF; монстр складывает полный signed delta
//! с maximum_attack modifier, а не подменяет итоговый getter. Временное +0x3C
//! сериализатором не читается и представлено локальным скаляром.
//! DecodeExStates назначает sufferer до Begin; отказ NULL Begin не запрещает
//! формулу загруженного игрока, но не создаёт visual.

use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::appserver::moveshape::StateKey;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape, timed_client_state_time};
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const FURY_STATE_SKILL_ID: u32 = 0x1a3;
pub(crate) const FURY_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FuryState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_gain_percent: i32,
}

impl FuryState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        attack_gain_percent: i32,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            attack_gain_percent,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        FURY_STATE_SKILL_ID
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != FURY_STATE_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?))
    }


    pub(crate) fn encoded_for_install(self) -> [u8; FURY_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; FURY_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; FURY_STATE_BYTES] {
        let mut bytes = [0; FURY_STATE_BYTES];
        bytes[..4].copy_from_slice(&FURY_STATE_SKILL_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..].copy_from_slice(&self.attack_gain_percent.to_le_bytes());
        bytes
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    fn truncated_gain(self, maximum: u32) -> i32 {
        truncate_original_i64_low(
            f64::from(self.attack_gain_percent)
                * f64::from(0.01_f32)
                * f64::from(maximum),
        )
    }

    pub(crate) fn apply_to_player_maximum_attack(self, maximum: u32) -> u32 {
        let mut gain = self.truncated_gain(maximum) as u16 as u32;
        if maximum.wrapping_add(gain) > u16::MAX as u32 {
            gain = (u16::MAX as u32).wrapping_sub(maximum);
        }
        maximum.wrapping_add(gain as u16 as u32).min(i32::MAX as u32)
    }
}




pub(crate) fn update_fury_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<FuryState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<FuryState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        let maximum = game.find_region(target_region)
            .and_then(|region| region.base().find_monster_by_id(target.id))
            .and_then(|monster| {
                let property = game.find_monster_property_by_origin_name(monster.original_name())?;
                Some(monster.state_attack_bounds(property.minimum_attack, property.maximum_attack).1)
            });
        if let Some(maximum) = maximum {
            let gain = state.truncated_gain(maximum);
            if let Some(monster) = game.find_region_mut(target_region)
                .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
                let modifiers = monster.move_shape_mut().property_modifiers_mut();
                modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(gain);
            }
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.maximum_attack = state.apply_to_player_maximum_attack(properties.maximum_attack);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_fury_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn update_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<FuryState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_fury_state(game, region_id, holder, key)
}

pub(crate) fn end_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<FuryState>(key)).copied()
        else { return false };
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_was_loaded(key)) == Some(false) {
        let mut message = CMessage::new(0x000b_fe04);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        let _ = game.send_move_shape_around(region_id, holder, &message);
    }
    end_base_applied_state(game, region_id, holder, key, FURY_STATE_BYTES)
}



// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp

// ============================================================================
// FUNCTION: CFuryState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:156
// RVA: 0x001FD660
// ADDRESS: 005fd660
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
// COMPONENT_VARIANT_END: GameServer
