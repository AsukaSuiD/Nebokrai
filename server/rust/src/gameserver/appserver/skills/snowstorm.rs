//! Призыв снежной бури CSnowStorm.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snowstorm.cpp.
//! Общий player/monster cast находится в zonalcast. Summon сохраняет Master
//! с country0 и Player EM либо0 до свежей таблицы. Порядок запросов
//! хранится в zone/skills/snowstorm.rs; здесь остаются запросы к CGame.
//! После свойств ctor получает часы и ID.
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
use nebokrai_zone::skills::SnowStormSummonParameters;

pub(crate) use nebokrai_zone::skills::SNOW_STORM_SKILL_ID;

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
    let Some(parameters) = SnowStormSummonParameters::read(
        |property| properties.query_property(property),
        || game.registered_skill(instance).map(|skill| skill.level()),
    ) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = match CSnowStormPhalanx::new(
        id, master, started, element, parameters,
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
