//! Каноническое периодическое состояние семейства `CHealState`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/healstate.cpp`. Один проход `AI` даёт не более одного
//! лечения, даже если пропущено несколько интервалов. Счётчик увеличивается
//! перед расчётом, прибавление HP использует DWORD-обёртку, а коэффициент
//! `Promotion` считывается заново при каждом проходе. Визуальные пакеты
//! `0xBFE03/0xBFE04` формируются в модуле-владельце состояния. Фактическая
//! публикация изменённого HP использует точный базовый `OnChangeStates` цели:
//! `DWORD HP, DWORD MP, WORD RP, WORD YP` и текущий spatial owner. Цель эффекта
//! хранится отдельно от владельца записи только ради подтверждённой
//! ветви `CSuperHeal2`; DB-запись её не сохраняет, поэтому после загрузки
//! целью снова становится владелец записи, как в исходном `Unserialize`.
//! Vtable exact EXE направляет клиентский срок семейства на общее тело
//! `CBlindState::GetRemainedTime` по `0x005F2CD0`.
//! Периодическая прибавка загружает полный unsigned `hp_gain` в x87,
//! умножает на сохранённый `f32` Promotion-множитель и усекается `FISTP dword`
//! без промежуточного `f32` и без округления дробной части.

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
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
    pub(crate) ended: bool,
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
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let keep_time_ms = reader.read_u32()?;
        let frequency_ms = reader.read_u32()?;
        let hp_gain = reader.read_u32()?;
        Ok(Self::new(
            skill_id,
            effect_target,
            0,
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

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
        self.heal_count = 0;
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn advance(
        &mut self,
        now_ms: u32,
        mut health: u32,
        maximum_health: u32,
        dead: bool,
        promotion_factor: Option<u16>,
    ) -> HealStatePass {
        if dead {
            return HealStatePass {
                health,
                changed: false,
                ended: true,
            };
        }
        let due = self
            .started_at_ms
            .wrapping_add(self.frequency_ms.wrapping_mul(self.heal_count))
            < now_ms;
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
            ended: self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms,
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

fn take_stored_states(
    game: &mut CGame,
    region_id: i32,
    storage: ShapeIdentity,
) -> Option<Vec<HealState>> {
    match storage.object_type {
        400 => game
            .find_player_mut(storage.id)
            .filter(|player| player.server_region_id() == Some(region_id))
            .map(CPlayer::take_heal_states),
        600 => {
            let mut owner = game.take_region_owner(region_id)?;
            let states = owner
                .base_mut()
                .find_monster_by_id_mut(storage.id)
                .map(|monster| monster.move_shape_mut().take_heal_states());
            game.restore_region_owner(owner);
            states
        }
        _ => None,
    }
}

fn restore_stored_states(
    game: &mut CGame,
    region_id: i32,
    storage: ShapeIdentity,
    states: Vec<HealState>,
) {
    match storage.object_type {
        400 => {
            if let Some(player) = game
                .find_player_mut(storage.id)
                .filter(|player| player.server_region_id() == Some(region_id))
            {
                player.restore_heal_states(states);
            }
        }
        600 => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(storage.id) {
                    monster.move_shape_mut().restore_heal_states(states);
                }
                game.restore_region_owner(owner);
            }
        }
        _ => {}
    }
}

fn advance_effect(
    game: &mut CGame,
    region_id: i32,
    state: &mut HealState,
    now_ms: u32,
) -> Option<(bool, i32, i32)> {
    let target = state.effect_target();
    match target.object_type {
        400 => {
            let player = game
                .find_player_mut(target.id)
                .filter(|player| player.server_region_id() == Some(region_id))?;
            let tile_x = player.shape().get_tile_x().ok()?;
            let tile_y = player.shape().get_tile_y().ok()?;
            let pass = state.advance(
                now_ms,
                player.health(),
                player.maximum_health(),
                player.is_dead(),
                player.promotion_heal_recover_factor(),
            );
            player.set_health(pass.health);
            if pass.changed {
                let _ = game.publish_player_states(target.id);
            }
            Some((pass.ended, tile_x, tile_y))
        }
        600 => {
            let property = game
                .find_region(region_id)
                .and_then(|owner| owner.base().find_monster_by_id(target.id))
                .and_then(CMonster::base_property_key)
                .and_then(|key| game.find_monster_property_by_origin_name(key))
                .cloned()?;
            let mut owner = game.take_region_owner(region_id)?;
            let result = owner
                .base_mut()
                .find_monster_by_id_mut(target.id)
                .and_then(|monster| {
                    let tile_x = monster.move_shape().shape().get_tile_x().ok()?;
                    let tile_y = monster.move_shape().shape().get_tile_y().ok()?;
                    let maximum_health = monster.maximum_hp(&property);
                    let health = monster.hit_points();
                    let promotion = monster.move_shape().promotion_heal_recover_factor();
                    let pass = state.advance(
                        now_ms,
                        health,
                        maximum_health,
                        health == 0,
                        promotion,
                    );
                    monster.set_hit_points(pass.health);
                    Some((pass, tile_x, tile_y))
                });
            if result.as_ref().is_some_and(|(pass, _, _)| pass.changed) {
                let _ = game.publish_owned_monster_states(owner.base(), target.id);
            }
            game.restore_region_owner(owner);
            let (pass, tile_x, tile_y) = result?;
            Some((pass.ended, tile_x, tile_y))
        }
        _ => None,
    }
}

/// Выполняет один подтверждённый AI-такт всех состояний лечения, хранящихся
/// у игрока либо монстра. Временное изъятие сохраняет исходный порядок
/// состояний и позволяет эффекту обращаться к независимому владельцу цели;
/// исчезнувшая цель завершает только соответствующую запись.
pub(crate) fn update_stored_heal_states(
    game: &mut CGame,
    region_id: i32,
    storage: ShapeIdentity,
    now_ms: u32,
) {
    let Some(states) = take_stored_states(game, region_id, storage) else {
        return;
    };
    let mut active = Vec::with_capacity(states.len());
    let mut removed_skill_ids = Vec::new();
    for mut state in states {
        let target = state.effect_target();
        match advance_effect(game, region_id, &mut state, now_ms) {
            Some((true, tile_x, tile_y)) => {
                removed_skill_ids.push(state.skill_id());
                send_heal_state_visual(
                    game, region_id, target, tile_x, tile_y, state, false, || now_ms,
                );
            }
            Some((false, _, _)) => active.push(state),
            None => removed_skill_ids.push(state.skill_id()),
        }
    }
    restore_stored_states(game, region_id, storage, active);
    match storage.object_type {
        400 => {
            if let Some(player) = game.find_player_mut(storage.id) {
                player.remove_serialized_heal_states(&removed_skill_ids);
            }
        }
        600 => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(storage.id) {
                    monster
                        .move_shape_mut()
                        .remove_serialized_heal_states(&removed_skill_ids);
                }
                game.restore_region_owner(owner);
            }
        }
        _ => {}
    }
}
