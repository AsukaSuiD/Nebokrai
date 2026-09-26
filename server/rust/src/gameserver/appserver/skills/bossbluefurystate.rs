//! Тонкий путь `CBossBlueFuryState` (`0x1F7`) в Zone.
//! Источник: gameserver.exe + GameServer.pdb, исходный владелец
//! `appserver/skills/bossbluefurystate.cpp/.h` (ctor `0x5E8A60`, Begin
//! `0x5E8ED0`, AI `0x5E8D50`, End `0x5E8D10`, Restart `0x5FD450`,
//! OnUpdateProperties `0x5E8DC0`). Живые callbacks и установочный Begin
//! перенесены буквально в `nebokrai_zone::skills::bossbluefurystate`
//! (статусы и швы — там и в `.local/recon-de/notes/E4-bossbluefury.md`);
//! данные/кодек — `nebokrai_zone::effects/bossbluefury`. Здесь — фасадные
//! реализации `BossBlueFuryStateGame` (origin-name снимок, живые границы и
//! две прибавки модификаторов монстра) и делегации арены с прежними
//! сигнатурами; обход состояний (`states/state.rs`) и потребители не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::skills::bossbluefurystate::BossBlueFuryStateGame;

pub(crate) use nebokrai_zone::effects::BossBlueFuryState;

impl BossBlueFuryStateGame for CGame {
    fn boss_blue_fury_monster_attack_base(
        &self,
        region_id: i32,
        monster_id: i32,
    ) -> Option<(u32, u32)> {
        let monster = self.find_region(region_id)?.base().find_monster_by_id(monster_id)?;
        let property = self.find_monster_property_by_origin_name(monster.original_name())?;
        Some((property.minimum_attack, property.maximum_attack))
    }

    fn boss_blue_fury_monster_attack_bounds(
        &self,
        region_id: i32,
        monster_id: i32,
        base: (u32, u32),
    ) -> Option<(u32, u32)> {
        let monster = self.find_region(region_id)?.base().find_monster_by_id(monster_id)?;
        Some(monster.state_attack_bounds(base.0, base.1))
    }

    fn apply_boss_blue_fury_monster_gains(
        &mut self,
        region_id: i32,
        monster_id: i32,
        minimum_gain: i32,
        maximum_gain: i32,
    ) {
        if let Some(monster) = self.find_region_mut(region_id)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
        {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.minimum_attack = modifiers.minimum_attack.wrapping_add(minimum_gain);
            modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(maximum_gain);
        }
    }
}

pub(crate) fn update_boss_blue_fury_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::bossbluefurystate::update_boss_blue_fury_state_properties(
        game, region_id, holder, key, now,
    )
}

pub(crate) fn restart_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::bossbluefurystate::restart_boss_blue_fury_state(
        game, region_id, holder, key, changing_region, now,
    )
}

pub(crate) fn update_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_milliseconds: impl FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::bossbluefurystate::update_boss_blue_fury_state(
        game, region_id, holder, key, now_milliseconds,
    )
}

pub(crate) fn end_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    nebokrai_zone::skills::bossbluefurystate::end_boss_blue_fury_state_key(
        game, region_id, holder, key,
    )
}
