//! Каноническое периодическое состояние семейства `CHealState`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/healstate.cpp`. Один проход `AI` каждого экземпляра даёт
//! не более одного лечения, даже если пропущено несколько интервалов.
//! Счётчик увеличивается перед расчётом, прибавление HP использует
//! DWORD-обёртку, а коэффициент `Promotion` считывается заново при каждом
//! проходе. Визуальные пакеты
//! `0xBFE03/0xBFE04` формируются в модуле-владельце состояния. Фактическая
//! публикация изменённого HP использует точный базовый `OnChangeStates` цели:
//! `DWORD HP, DWORD MP, WORD RP, WORD YP` и текущий spatial owner. Цель эффекта
//! хранится отдельно от владельца записи только ради подтверждённой
//! ветви `CSuperHeal2`; DB-запись её не сохраняет, поэтому после загрузки
//! целью снова становится владелец записи при исходном `Begin(NULL, holder)`.
//! Vtable exact EXE направляет клиентский срок семейства на общее тело
//! `CBlindState::GetRemainedTime` по `0x005F2CD0`.
//! Периодическая прибавка загружает полный unsigned `hp_gain` в x87,
//! умножает на сохранённый `f32` Promotion-множитель и усекается `FISTP dword`
//! без промежуточного `f32` и без округления дробной части.
//! End (`0x005EEBA0`) не вызывает visual: он ищет собственную запись
//! в контейнере sufferer через RemoveState (`0x004CDAB0`). Только найденная
//! запись удаляется с UpdateProperty. При раздельных storage/target либо
//! исчезнувшей цели End ничего не меняет; состояние продолжает существовать.
//! Во время лечения все записи, включая текущее состояние, остаются у owner-а;
//! счётчик изменяется на месте по поколенческому ключу до публикации HP.
//! AI (`0x005EEDF0`) читает часы отдельно для интервала и для срока после
//! OnChangeStates; время общего прохода не заменяет эти два чтения. Мёртвая
//! цель завершает состояние до обращения к часам.
//! Общий CMoveShape::UpdateAbnormality передаёт один ключ; этот owner
//! не запускает отдельный семейный обход и сохраняет границы своих callbacks.
//! Прямой End и истечение используют один exact-key хвост без часов;
//! ключ никогда не ищется в чужой арене при отличающемся effect_target.

//! Restart воспроизводит только Begin(NULL, holder) (0x005F8BA0/0x005EFD20/0x005F6650/0x005EEBE0):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Begin создаёт принадлежащий записи loop=1 visual без немедленного пакета.
//! Sufferer перепривязывается к holder до visual; heal_count обнуляется после BeginVisual.

//! Unserialize 0x005EEC70 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

//! Vtable четырёх вариантов 0x00660D9C/0x0066024C/0x00660A64/0x00660134
//! в слоте +0x24 указывают на 0x0047B150: OnUpdateProperties возвращает 1
//! без target lookup, visual и часов. Лечение остаётся только у AI.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual,
};
use crate::gameserver::appserver::moveshape::StateKey;

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const HEAL_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const HEAL_STATE_END_MESSAGE: i32 = 0x000b_fe04;
pub(crate) const HEAL_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HealState {
    skill_id: u32,
    effect_target: ShapeIdentity,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_gain: u32,
    heal_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HealStatePass {
    pub(crate) health: u32,
    pub(crate) changed: bool,
    pub(crate) dead: bool,
}

impl HealState {
    pub(crate) const fn new(
        skill_id: u32,
        effect_target: ShapeIdentity,
        started_at_ms: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        hp_gain: u32,
    ) -> Self {
        Self {
            skill_id,
            effect_target,
            started_at_ms,
            keep_time_ms,
            frequency_ms,
            hp_gain,
            heal_count: 0,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub(crate) const fn effect_target(self) -> ShapeIdentity {
        self.effect_target
    }

    pub(crate) fn decode(
        payload: &[u8],
        offset: usize,
        effect_target: ShapeIdentity,
        now_ms: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let keep_time_ms = reader.read_u32()?;
        let frequency_ms = reader.read_u32()?;
        let hp_gain = reader.read_u32()?;
        Ok(Self::new(
            skill_id,
            effect_target,
            now_ms,
            keep_time_ms,
            frequency_ms,
            hp_gain,
        ))
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; HEAL_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    pub(crate) fn encoded_for_install(self) -> [u8; HEAL_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; HEAL_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(HEAL_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.skill_id);
        writer.write_u32(remaining_time_ms);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.hp_gain);
        bytes.try_into().expect("размер состояния лечения фиксирован")
    }



    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn advance(
        &mut self,
        mut now_milliseconds: impl FnMut() -> u32,
        mut health: u32,
        maximum_health: u32,
        dead: bool,
        promotion_factor: Option<u16>,
    ) -> HealStatePass {
        if dead {
            return HealStatePass {
                health,
                changed: false,
                dead: true,
            };
        }
        let due = self
            .started_at_ms
            .wrapping_add(self.frequency_ms.wrapping_mul(self.heal_count))
            < now_milliseconds();
        let mut changed = false;
        if due {
            self.heal_count = self.heal_count.wrapping_add(1);
            let multiplier = promotion_factor.map_or(1.0, |factor| f32::from(factor) * 0.001);
            let gain = truncate_original(
                f64::from(self.hp_gain) * f64::from(multiplier),
            ) as u32;
            let next = health.wrapping_add(gain).min(maximum_health);
            // Исходный `AI` вызывает `OnChangeStates` на каждом сработавшем
            // интервале, даже когда ограничение максимума оставило HP прежним.
            changed = true;
            health = next;
        }
        HealStatePass {
            health,
            changed,
            dead: false,
        }
    }
}

pub(crate) fn send_heal_state_visual(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: HealState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin {
        HEAL_STATE_BEGIN_MESSAGE
    } else {
        HEAL_STATE_END_MESSAGE
    });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

fn target_heal_values(
    game: &CGame,
    region_id: i32,
    target: ShapeIdentity,
) -> Option<(u32, u32, bool, Option<u16>)> {
    match target.object_type {
        400 => {
            let player = game.find_player(target.id)
                .filter(|player| player.server_region_id() == Some(region_id))?;
            player.shape().get_tile_x().ok()?;
            player.shape().get_tile_y().ok()?;
            Some((player.health(), player.maximum_health(), player.is_dead(),
                player.promotion_heal_recover_factor()))
        }
        600 => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(target.id)?;
            monster.move_shape().shape().get_tile_x().ok()?;
            monster.move_shape().shape().get_tile_y().ok()?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some((monster.hit_points(), monster.maximum_hp(property), monster.hit_points() == 0,
                monster.move_shape().promotion_heal_recover_factor()))
        }
        _ => None,
    }
}

pub(crate) fn restart_heal_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none() {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<HealState>(key)) {
        state.effect_target = holder;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<HealState>(key)) {
        state.heal_count = 0;
    }
    true
}

/// Счётчик меняется в живом payload до OnChangeStates. Между публикацией HP
/// и End экземпляр перечитывается по тому же ключу, без возврата старого снимка.
pub(crate) fn update_stored_heal_state(
    game: &mut CGame,
    region_id: i32,
    storage: ShapeIdentity,
    key: StateKey,
    mut now_milliseconds: impl FnMut() -> u32,
) {
    if !matches!(storage.object_type, 400 | 600) {
        return;
    }
    let Some(target) = resolve_state_move_shape(game, region_id, storage)
        .and_then(|shape| shape.applied_state::<HealState>(key))
        .map(|state| state.effect_target()) else { return };
    let Some((health, maximum_health, dead, promotion)) = target_heal_values(game, region_id, target)
        else { return };
    let Some(pass) = resolve_state_move_shape_mut(game, region_id, storage)
        .and_then(|shape| shape.applied_state_mut::<HealState>(key))
        .map(|state| state.advance(&mut now_milliseconds, health, maximum_health, dead, promotion))
        else { return };
    match target.object_type {
        400 => {
            let Some(player) = game.find_player_mut(target.id)
                .filter(|player| player.server_region_id() == Some(region_id)) else { return };
            player.set_health(pass.health);
            if pass.changed {
                let _ = game.publish_player_states(target.id);
            }
        }
        600 => {
            let Some(monster) = game.find_region_mut(region_id)
                .and_then(|owner| owner.base_mut().find_monster_by_id_mut(target.id))
                else { return };
            monster.set_hit_points(pass.health);
            if pass.changed && let Some(owner) = game.find_region(region_id) {
                let _ = game.publish_owned_monster_states(owner.base(), target.id);
            }
        }
        _ => return,
    }
    let Some(state) = resolve_state_move_shape(game, region_id, storage)
        .and_then(|shape| shape.applied_state::<HealState>(key)) else { return };
    let ended = pass.dead
        || state.started_at_ms.wrapping_add(state.keep_time_ms) < now_milliseconds();
    if ended {
        let _ = end_heal_state(game, region_id, storage, key);
    }
}

pub(crate) fn end_heal_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(target) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key))
        .map(|state| state.effect_target()) else { return false };
    if target.object_type != holder.object_type || target.id != holder.id {
        return false;
    }
    let removed = resolve_state_move_shape_mut(game, region_id, target)
        .and_then(|shape| shape.remove_heal_state_key(key)).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}
