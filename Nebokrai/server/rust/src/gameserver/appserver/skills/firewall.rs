//! Призыв огненной стены CFireWall.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/firewall.cpp.
//! Begin/Check/AI/visual/End общие в zonalcast; Check запрещает BLOCK1|2.
//! Summon сохраняет Master(country0)/Player EM, затем читает свежую таблицу.
//! После usage20015/FISTP: CONST×scaledEM+100 с DWORD wrapping→unsigned
//! коэффициент×0.01f→отдельный f32→LIFETIME/FISTP. Оба усечения предшествуют
//! CCH WORD→GetAddElementAttack→MAX→MIN→FREQUENCY→свежему уровню→ctor(clock→ID).
//! SetTile→свежий actual region U→Add→FindAroundObject(SUMMON_SHAPE_TYPE)
//! →Replace каждой стены со свежим уровнем навыка и исходными X/Y→encode/BF502.
//! Результат Add не отменяет публикацию, а Summon не определяет аргумент End.

use super::fightdefense::truncate_original;
use super::firewallphalanx::new_fire_wall_phalanx;
use super::weaponattack::{SourceProperty, source_property};
use super::zonalcast::prepare_element_summon;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const FIRE_WALL_SKILL_ID: u32 = 0x134;

pub(super) fn summon_fire_wall<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some((master, properties, scaled_element)) = prepare_element_summon(game, instance, source) else { return; };
    let constant = properties.query_property(20_010);
    let scale = constant.wrapping_mul(scaled_element as u32).wrapping_add(100);
    let factor = (f64::from(scale) * f64::from(0.01_f32)) as f32;
    let lifetime = properties.query_property(30_001);
    let lifetime = truncate_original(f64::from(lifetime) * f64::from(factor)) as u32;
    let Some(cch) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
    let cch = i32::from(cch as u16);
    let Some(element) = source_property(game, source, SourceProperty::Element) else { return; };
    let element = (element as i32).wrapping_add(scaled_element);
    let maximum = properties.query_property(20_009) as i32;
    let minimum = properties.query_property(20_008) as i32;
    let frequency = properties.query_property(6_001);
    let Some(level) = game.registered_skill(instance).map(|skill| i32::from(skill.level())) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = new_fire_wall_phalanx(
        id, master, started, lifetime, level, frequency, minimum, maximum, element, cch,
    );
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(destination.0) + 0.5) as f32, (f64::from(destination.1) + 0.5) as f32,
    );
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.add_masked_element_phalanx(region, phalanx, started, runtime, Some((instance, destination)));
}
