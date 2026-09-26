//! Тонкий путь формы `CThunderSlashPhalanx` в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/thunderslashphalanx.cpp.
//! Данные формы, часы, цель, Calculate (FIX #1: `info[+0] := instance-id`)
//! и Attack перенесены буквально в `nebokrai_zone::skills::thunderslashphalanx`
//! (основание и статусы см. там). Здесь — фасадные реализации hub-трейтов
//! Zone над прежними методами `CGame`/`CPlayer`; региональные потребители
//! (`serverregion.rs`, `game/thunderslash.rs`, enum `SummonedSkillShape`)
//! не меняются.

use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, RegionShapeResolver,
};
use nebokrai_zone::skills::thunderslashphalanx::{
    ThunderSlashPhalanxContact, ThunderSlashPhalanxGame, ThunderSlashPhalanxPlayer,
};

pub(crate) use nebokrai_zone::skills::thunderslashphalanx::{
    CThunderSlashPhalanx, apply_thunder_slash_attack, thunder_slash_target,
};

impl ThunderSlashPhalanxPlayer for CPlayer {
    fn shape(&self) -> &CShape { self.shape() }
}

impl ThunderSlashPhalanxGame for CGame {
    type Player = CPlayer;

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> { self.find_player(player_id) }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn skill_random_below(&mut self, maximum: i32) -> i32 { self.skill_random_below(maximum) }

    fn thunder_slash_critical_rate(&self) -> f32 { self.globe_setup().critical_rate() }

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool {
        self.base_magic_target_dead(region_id, target)
    }

    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        self.live_skill_target_attackable(region_id, source, target)
    }

    fn thunder_slash_target_present(&self, region_id: i32, target: ShapeIdentity) -> bool {
        resolve_state_move_shape(self, region_id, target).is_some()
    }

    fn increase_owned_player_rp(&mut self, player_id: i32, attacking: bool, damage: u16) {
        self.increase_owned_player_rp(player_id, attacking, damage)
    }

    fn thunder_slash_region_cell_target(
        &self,
        region_id: i32,
        x: i32,
        y: i32,
    ) -> Option<ShapeIdentity> {
        let owner = self.find_region(region_id)?;
        let resolver = RegionShapeResolver { game: self, owner };
        let (width, height) = self.area_dimensions();
        let first = owner.base().get_shape(x, y, width, height, &resolver).ok()??;
        resolve_state_move_shape(self, region_id, first.identity)?;
        Some(first.identity)
    }
}

impl<Runtime: GameMainLoopRuntime> ThunderSlashPhalanxContact<Runtime> for CGame {
    fn apply_owned_skill_contact(
        &mut self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_contact(master, target, region_id, attack, runtime)
    }
}
