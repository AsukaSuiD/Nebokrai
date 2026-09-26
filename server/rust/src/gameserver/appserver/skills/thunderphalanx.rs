//! Тонкий путь к живой области CThunderPhalanx в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/thunderphalanx.cpp. Живое тело области (форма, окна,
//! тики, wire и calc) перенесено буквально в
//! `nebokrai_zone::skills::thunderphalanx` порцией T2 «BF-облака области»
//! (истинные RVA-якоря и статусы MATCH/PARTIAL — в шапке Zone-файла).
//! Здесь — реэкспорт прежних имён и фасадная реализация hub-трейта
//! `ThunderPhalanxGame` (оружейные чтения прежнего `CGame`) для zone-calc.
//! Потребители (game run-flow, serverregion, summonshape, thunder) не
//! меняются.

use nebokrai_zone::skills::ThunderPhalanxGame;

use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::skills::{
    CThunderPhalanx, ThunderPhalanxTick, calculate_owned_thunder_attack,
};

impl ThunderPhalanxGame for CGame {
    fn thunder_phalanx_weapon_damage_factors(&self) -> (f32, f32) {
        self.globe_setup().weapon_damage_factors()
    }

    fn thunder_phalanx_weapon_modifier(
        &self,
        player_id: i32,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> Option<f32> {
        let player = self.find_player(player_id)?;
        Some(player.weapon_modifier(self.goods_factory(), target_level, divisor, minimum_factor))
    }
}
