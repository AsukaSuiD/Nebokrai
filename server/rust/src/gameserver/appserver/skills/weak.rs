//! Призыв области ослабления CWeak.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/weak.cpp.
//! Общий Begin/Check/AI/visual/End находится в zonalcast. Summon сначала
//! сохраняет actual region U и отвергает GetSecurity==SAFE, не GetBlock.
//! Master(country0)/Player EM либо0 предшествуют свежей таблице.
//! Расчёт срока CONST×EM+100 и FISTP находится в zone/skills/weak.rs.
//! ATTACK_LOSS→свежий уровень→ctor(clock→ID) создают область 1×1.
//! SetTile→перекрытие старых областей→Add→encode/BF502 сохраняют ранний регион;
//! отказ Add не отменяет сериализацию. Отказ Summon не меняет End1 навыка.

use super::weakphalanx::CWeakPhalanx;
use super::weaponattack::source_master;
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::weak_lifetime;

pub(crate) use nebokrai_zone::skills::WEAK_SKILL_ID;
const ATTACK_LOSS: u32 = 205;
const LIFETIME_FACTOR: u32 = 20_010;
const LIFETIME: u32 = 30_001;

pub(super) fn summon_weak<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    let Some(region) = game.find_region(region_id) else { return; };
    if region.get_security(destination.0, destination.1).ok() == Some(RegionSecurity::SAFE) { return; }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let element = if source.1.object_type == 400 {
        game.find_player(source.1.id).map_or(0, |player| player.combat_properties().element_modify as u32)
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let lifetime = weak_lifetime(
        properties.query_property(LIFETIME_FACTOR), element, properties.query_property(LIFETIME),
    );
    let attack_loss = properties.query_property(ATTACK_LOSS);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CWeakPhalanx::new(id, master, started, lifetime, level, 1, 1, attack_loss);
    phalanx.set_center(destination.0, destination.1);
    if let Some(Ok(id)) = game.add_weak_phalanx(region_id, phalanx, destination.0, destination.1, started, runtime) {
        let _ = game.send_weak_phalanx_entry(region_id, id, runtime);
    }
}
