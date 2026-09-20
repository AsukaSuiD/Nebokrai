//! Ослабление защиты и сопротивления ядовитым туманом (0xC9).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/poisonfogstate.cpp.
//!
//! Общая арена владеет экземпляром, visual и DB-span. Begin требует S,
//! создаёт loop1 visual без Update; перезапуск с NULL U сохраняет timestamp.
//! End выполняет visual, затем удаляет тот же экземпляр у заново найденного S.
//! Неиспользуемые координаты и CScope состояния не материализованы: срок
//! определяется только часами, а геометрия принадлежит создающей области.
//! Поле dodge_loss сохраняется в 36-байтовой записи, но формулы его не читают.
//! Load читает часы после уровня и до remaining; Save не меняет payload.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, remove_applied_state_from,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    timed_client_state_time, update_applied_state_end_visual, update_property_state_visual,
    StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

pub(crate) const POISON_FOG_STATE_ID: u32 = 0xc9;
pub(crate) const POISON_FOG_STATE_BYTES: usize = 36;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PoisonFogState {
    skill_level: i32,
    started_at_ms: u32,
    keep_time_ms: u32,
    defense_loss: u32,
    defense_loss_coefficient: u32,
    dodge_loss: u32,
    element_resistance_loss: u32,
    element_resistance_loss_coefficient: u32,
    weapon_damage_level: u32,
}

impl PoisonFogState {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют состоянию EXE")]
    pub(crate) const fn new(
        skill_level: i32,
        keep_time_ms: u32,
        defense_loss: u32,
        defense_loss_coefficient: u32,
        dodge_loss: u32,
        element_resistance_loss: u32,
        element_resistance_loss_coefficient: u32,
        weapon_damage_level: u32,
    ) -> Self {
        Self {
            skill_level, started_at_ms: 0, keep_time_ms,
            defense_loss, defense_loss_coefficient, dodge_loss,
            element_resistance_loss, element_resistance_loss_coefficient,
            weapon_damage_level,
        }
    }

    pub(crate) const fn skill_id(&self) -> u32 { POISON_FOG_STATE_ID }

    pub(crate) const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }

    // У игрока разница уровней остаётся в x87; произведение и ограничение
    // текущим свойством проходят через float, затем усечение до младшего WORD.
    fn player_loss(&self, target_level: u8, coefficient: u32, maximum: u32, current: u32) -> u32 {
        let scaled = if coefficient == 0 {
            0.0
        } else {
            let difference = f64::from(self.weapon_damage_level) - f64::from(target_level);
            let ratio = (difference / f64::from(coefficient)).clamp(0.0, 1.0);
            (f64::from(maximum) * ratio) as f32
        };
        let capped = if f64::from(current) < f64::from(scaled) {
            f64::from(current) as f32
        } else {
            scaled
        };
        truncate_original(f64::from(capped)) as u32 & 0xffff
    }

    // У монстра сама разница уже float. Произведение защиты остаётся в x87,
    // а произведение сопротивления сначала записывается во float.
    fn monster_losses(&self, target_level: u8) -> (u32, u32) {
        let difference = (f64::from(self.weapon_damage_level) - f64::from(target_level)) as f32;
        let ratio = |coefficient: u32| {
            if coefficient == 0 {
                0.0
            } else {
                (f64::from(difference) / f64::from(coefficient)).clamp(0.0, 1.0)
            }
        };
        let defense = truncate_original(
            f64::from(self.defense_loss) * ratio(self.defense_loss_coefficient),
        ) as u32;
        let resistance = (f64::from(self.element_resistance_loss)
            * ratio(self.element_resistance_loss_coefficient)) as f32;
        (defense, truncate_original(f64::from(resistance)) as u32)
    }

    pub(crate) fn apply_to_player(&self, target_level: u8, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        let defense = self.player_loss(
            target_level,
            self.defense_loss_coefficient,
            self.defense_loss,
            properties.defense,
        );
        let resistance = self.player_loss(
            target_level,
            self.element_resistance_loss_coefficient,
            self.element_resistance_loss,
            properties.element_resistance,
        );
        properties.defense = properties.defense.wrapping_sub(defense).min(i32::MAX as u32);
        properties.element_resistance = properties.element_resistance.wrapping_sub(resistance).min(i32::MAX as u32);
        properties
    }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != POISON_FOG_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let skill_level = reader.read_i32()?;
        let started_at_ms = now();
        Ok(Self {
            skill_level, started_at_ms,
            keep_time_ms: reader.read_u32()?,
            defense_loss: reader.read_u32()?,
            defense_loss_coefficient: reader.read_u32()?,
            dodge_loss: reader.read_u32()?,
            element_resistance_loss: reader.read_u32()?,
            element_resistance_loss_coefficient: reader.read_u32()?,
            weapon_damage_level: reader.read_u32()?,
        })
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; POISON_FOG_STATE_BYTES] {
        self.encode_record(|| self.client_time(now) as u32)
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; POISON_FOG_STATE_BYTES] {
        self.encode_record(|| self.keep_time_ms)
    }

    fn encode_record(&self, remaining: impl FnOnce() -> u32) -> [u8; POISON_FOG_STATE_BYTES] {
        let mut record = Vec::with_capacity(POISON_FOG_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut record);
        writer.write_u32(POISON_FOG_STATE_ID);
        writer.write_i32(self.skill_level);
        writer.write_u32(remaining());
        writer.write_u32(self.defense_loss);
        writer.write_u32(self.defense_loss_coefficient);
        writer.write_u32(self.dodge_loss);
        writer.write_u32(self.element_resistance_loss);
        writer.write_u32(self.element_resistance_loss_coefficient);
        writer.write_u32(self.weapon_damage_level);
        record.try_into().expect("размер записи ядовитого тумана фиксирован")
    }
}

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(crate) fn begin_primary_poison_fog_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: PoisonFogState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    resolve_state_move_shape(game, holder_region, holder)?;
    resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    if user.is_some() { state.started_at_ms = now(); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}


pub(crate) fn update_poison_fog_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    let Some((target_region, target)) = resolve_applied_state_sufferer(
        game, region_id, holder, key,
    ) else { return false };
    let target_level = match target.object_type {
        400 => {
            let Some(player) = game.find_player(target.id) else { return false };
            player.level()
        }
        600 => {
            let Some(monster) = game.find_region(target_region)
                .and_then(|region| region.base().find_monster_by_id(target.id))
            else { return false };
            let Some(properties) = monster.base_property_key()
                .and_then(|name| game.find_monster_property_by_origin_name(name))
            else { return false };
            properties.level as u8
        }
        500 | 1100 | 1200 => 1,
        _ => return false,
    };
    let _ = update_property_state_visual::<PoisonFogState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key))
    else { return false };
    match target.object_type {
        400 => {
            let Some(properties) = game.find_player(target.id).map(|player| player.combat_properties())
            else { return false };
            let updated = state.apply_to_player(target_level, properties);
            let Some(player) = game.find_player_mut(target.id) else { return false };
            player.update_state_combat_properties(|_| updated);
        }
        600 => {
            let (defense, resistance) = state.monster_losses(target_level);
            let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
            else { return false };
            let modifiers = shape.property_modifiers_mut();
            modifiers.defense = modifiers.defense.wrapping_sub(defense as i32);
            modifiers.element_resistance = modifiers.element_resistance
                .wrapping_sub(resistance as i32);
        }
        _ => {}
    }
    true
}

pub(crate) fn restart_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(
        game, region_id, holder, key,
    ) { return false }
    let _ = begin_applied_state_visual(
        game, region_id, holder, key, 1,
    );
    true
}

pub(crate) fn update_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key))
        .is_some_and(|state| state.expired(now_ms));
    if !expired { return false }
    end_poison_fog_state(game, region_id, holder, key)
}

pub(crate) fn end_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    let Some((target_region, target)) = resolve_applied_state_sufferer(
        game, region_id, holder, key,
    ) else { return false };
    remove_applied_state_from(
        game, region_id, holder, key, (target_region, target), POISON_FOG_STATE_BYTES,
    )
}

// Для координатного Begin (0x00607E30) и typed Begin (0x00607EC0)
// вызывающие цепочки не установлены; основной путь использует объектный Begin.
