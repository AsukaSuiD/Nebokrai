//! Тонкий путь `CRageBreakState` (0x6E) в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/ragebreakstate.cpp.
//! Живые Begin/End/Restart/AI и пересчёт перенесены буквально в
//! `nebokrai_zone::skills::ragebreakstate` (машинные якоря `0x5FD1C0`…
//! `0x605E10`, статусы и швы — там); consume ThunderSlash выполняется
//! внутри Zone. Здесь — фасадные реализации AttackGain-швов Zone над
//! прежними методами `CGame` и делегации callbacks арены с прежними
//! сигнатурами; обход состояний (`states/state.rs`) и Fury не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::skills::ragebreakstate::AttackGainStateGame;

pub(crate) use nebokrai_zone::effects::{RAGE_BREAK_STATE_ID, RageBreakState};

impl AttackGainStateGame for CGame {
    fn attack_gain_monster_maximum(&self, region_id: i32, monster_id: i32) -> Option<u32> {
        let monster = self.find_region(region_id)?.base().find_monster_by_id(monster_id)?;
        let property = self.find_monster_property_by_origin_name(monster.original_name())?;
        Some(monster.state_attack_bounds(property.minimum_attack, property.maximum_attack).1)
    }

    fn apply_attack_gain_monster_maximum(&mut self, region_id: i32, monster_id: i32, gain: i32) {
        if let Some(monster) = self.find_region_mut(region_id)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
        {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(gain);
        }
    }
}

pub(crate) fn end_rage_break_state_key(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    nebokrai_zone::skills::ragebreakstate::end_rage_break_state_key(game, region_id, holder, key)
}

pub(crate) fn update_rage_break_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    key: StateKey, now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::ragebreakstate::update_rage_break_state_properties(
        game, region_id, holder, key, now,
    )
}

pub(crate) fn restart_rage_break_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::ragebreakstate::restart_rage_break_state(
        game, region_id, holder, key, changing_region, now,
    )
}

pub(crate) fn update_rage_break_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    nebokrai_zone::skills::ragebreakstate::update_rage_break_state(
        game, region_id, holder, key, now_ms,
    )
}
