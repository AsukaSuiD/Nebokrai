//! Делегат городского лучника `CCityGuardWithBow` (AI11) в Zone.
//!
//! Hurt-повторный поиск вне боя перенесён буквально в
//! `nebokrai_zone::ai::cityguardwithbow` — машинная база `MATCH` по точной
//! паре `4F5C98E0…` + GameServer.pdb (RSDS match), якорь (`0x0060DA90`
//! WhenBeenHurted: `SearchEnemyGuildMember` vt `+0x90` +
//! `SearchEnemyGuildPet` vt `+0x94` без вызова повозок `+0x98`) описан в её
//! шапке волной Z-AI. Здесь — прежняя сигнатура hurt-входа: hub-трейты
//! городской пары реализованы в делегате `cityguardwithsword`, потребитель
//! (`periodicattack`) не меняется.

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

/// `WhenBeenHurted` AI11: вне боя заново выполняется общий городской поиск
/// игроков и питомцев текущим навыком (прежняя сигнатура).
pub(crate) fn retarget_city_bow_guard_after_hurt(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    now_ms: u32,
) {
    nebokrai_zone::ai::cityguardwithbow::retarget_city_bow_guard_after_hurt(
        game, region, monster_id, property, now_ms,
    );
}
