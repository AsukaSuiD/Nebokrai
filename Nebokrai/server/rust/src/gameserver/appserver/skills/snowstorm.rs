//! Призыв снежной бури CSnowStorm.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snowstorm.cpp.
//! Общий player/monster cast находится в zonalcast. Summon сохраняет Master
//! с country0 и Player EM либо0 до свежей таблицы. Порядок запросов:
//! CONST→MAX→MIN→FREQUENCY→свежий уровень→LIFETIME→ctor(clock→ID).
//! SetTile→Initialize с RNG выполняются до повторного чтения actual region U;
//! Add→encode/BF502 не зависят от результата регистрации области.
//! Некорректные параметры, вызывающие native деление на ноль или выход из
//! массива, явно диагностируются конструктором и не создают успешную область.
//! Это безопасная граница, а не изменение валидной формулы или RNG-порядка.

use super::snowstormphalanx::CSnowStormPhalanx;
use super::weaponattack::source_master;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const SNOW_STORM_SKILL_ID: u32 = 0x193;
const TARGET_COUNT: u32 = 20_010;
const MINIMUM_ATTACK: u32 = 20_008;
const MAXIMUM_ATTACK: u32 = 20_009;
const FREQUENCY: u32 = 6_001;
const LIFETIME: u32 = 30_001;

pub(super) fn summon_snow_storm<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let element = if source.1.object_type == 400 {
        game.find_player(source.1.id).map_or(0, |player| player.combat_properties().element_modify)
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let count = properties.query_property(TARGET_COUNT);
    let maximum = properties.query_property(MAXIMUM_ATTACK) as i32;
    let minimum = properties.query_property(MINIMUM_ATTACK) as i32;
    let frequency = properties.query_property(FREQUENCY);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = match CSnowStormPhalanx::new(
        id, master, started, lifetime, level, frequency, minimum, maximum, element, count,
    ) {
        Ok(phalanx) => phalanx,
        Err(error) => {
            tracing::error!(skill_id = SNOW_STORM_SKILL_ID, ?error, "некорректные параметры снежной бури");
            return;
        }
    };
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(destination.0) + 0.5) as f32, (f64::from(destination.1) + 0.5) as f32,
    );
    phalanx.initialize(&mut |maximum| game.skill_random_below(maximum));
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    let _ = game.add_snow_storm_phalanx(region_id, phalanx, started, runtime);
}
