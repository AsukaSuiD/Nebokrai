//! Призыв огненной стены CFireWall.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/firewall.cpp.
//! Begin/Check/AI/visual/End общие в zonalcast; Check запрещает BLOCK1|2.
//! Summon сохраняет Master(country0)/Player EM, затем читает свежую таблицу.
//! Ключи, формула срока и порядок живых чтений принадлежат zone/skills/firewall.rs.
//! SetTile→свежий actual region U→Add→FindAroundObject(SUMMON_SHAPE_TYPE)
//! →Replace каждой стены со свежим уровнем навыка и исходными X/Y→encode/BF502.
//! Результат Add не отменяет публикацию, а Summon не определяет аргумент End.

use super::firewallphalanx::new_fire_wall_phalanx;
use super::weaponattack::{SourceProperty, source_property};
use super::zonalcast::prepare_element_summon;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::{FireWallLiveField, FireWallSummonParameters};

pub(crate) use nebokrai_zone::skills::FIRE_WALL_SKILL_ID;

pub(super) fn summon_fire_wall<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some((master, properties, scaled_element)) = prepare_element_summon(game, instance, source) else { return; };
    let Some(parameters) = FireWallSummonParameters::read(
        |property| properties.query_property(property),
        |field| match field {
            FireWallLiveField::CriticalChance => source_property(game, source, SourceProperty::CriticalChance).map(|value| value as i32),
            FireWallLiveField::AddElementAttack => source_property(game, source, SourceProperty::Element).map(|value| value as i32),
            FireWallLiveField::SkillLevel => game.registered_skill(instance).map(|skill| skill.level()),
        },
        scaled_element,
    ) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = new_fire_wall_phalanx(
        id, master, started, parameters.lifetime_ms, parameters.skill_level,
        parameters.frequency_ms, parameters.minimum_attack, parameters.maximum_attack,
        parameters.element_attack, parameters.critical_chance,
    );
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(destination.0) + 0.5) as f32, (f64::from(destination.1) + 0.5) as f32,
    );
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.add_masked_element_phalanx(region, phalanx, started, runtime, Some((instance, destination)));
}
