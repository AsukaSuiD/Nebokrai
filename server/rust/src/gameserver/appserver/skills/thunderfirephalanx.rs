//! Тонкий путь формы `CThunderFirePhalanx` (`0x322`) в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/thunderfirephalanx.cpp.
//! Данные формы, тик AI (FIX #2: `idx >= count` завершает форму только в
//! due-ветке с регионом), Calculate (`GetWeaponModifier `0x42D980`, x87
//! soul-формула) и клиентский снимок перенесены буквально в
//! `nebokrai_zone::skills::thunderfirephalanx` (основание, статусы и
//! UNKNOWN нормализации scale — там). Здесь — фасадные реализации
//! hub-трейтов Zone над прежними методами `CGame`/`CPlayer`; потребители
//! (`itemskill2.rs`, `serverregion.rs`, `game.rs`, enum `SummonedSkillShape`)
//! не меняются.

use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::combat::PlayerCombatProperties;
use nebokrai_zone::skills::thunderfirephalanx::{
    ThunderFirePhalanxGame, ThunderFirePhalanxPlayer,
};

pub(crate) use nebokrai_zone::skills::thunderfirephalanx::{
    CThunderFirePhalanx, ThunderFirePhalanxTick, calculate_owned_thunder_fire_attack,
};

impl ThunderFirePhalanxPlayer for CPlayer {
    fn combat_properties(&self) -> PlayerCombatProperties { self.combat_properties() }

    fn occupation(&self) -> u8 { self.occupation() }

    fn level(&self) -> u8 { self.level() }
}

impl ThunderFirePhalanxGame for CGame {
    type Player = CPlayer;

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> { self.find_player(player_id) }

    fn thunder_fire_weapon_modifier(
        &self,
        player: &CPlayer,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> f32 {
        player.weapon_modifier(self.goods_factory(), target_level, divisor, minimum_factor)
    }

    fn weapon_damage_factors(&self) -> (f32, f32) { self.globe_setup().weapon_damage_factors() }

    fn skill_random_below(&mut self, maximum: i32) -> i32 { self.skill_random_below(maximum) }
}
