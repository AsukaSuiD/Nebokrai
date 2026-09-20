//! Призыв ядовитого тумана CPoisonFog.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/poisonfog.cpp.
//! Общий cast находится в zonalcast. Summon захватывает полный U, Master
//! с country0 и уровень оружия Player либо0, затем свежую таблицу.
//! ER_COEFF→ER_LOSS→DODGE_LOSS→DEF_COEFF→DEF_LOSS→PERSIST→уровень→LIFETIME
//! предшествуют ctor(clock→ID). SetTile выполняется до свежего actual region U.
//! Перекрытие прежних C9 очищает их маску до Add; encode/BF502 остаются
//! обязательны даже при отказе Add. Состояния области не принадлежат cast-у.

use super::poisonfogphalanx::CPoisonFogPhalanx;
use super::weaponattack::source_master;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const POISON_FOG_SKILL_ID: u32 = 0xc9;
const STATE_TIME: u32 = 10_002;
const DEF_LOSS: u32 = 209;
const DODGE_LOSS: u32 = 210;
const ELEMENT_LOSS: u32 = 212;
const DEF_COEFFICIENT: u32 = 223;
const ER_COEFFICIENT: u32 = 224;
const LIFETIME: u32 = 30_001;

pub(super) fn summon_poison_fog<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let weapon = if source.1.object_type == 400 {
        game.find_player(source.1.id).map_or(0, |player| player.weapon_damage_level(game.goods_factory()) as u32)
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let er_coefficient = properties.query_property(ER_COEFFICIENT);
    let element_loss = properties.query_property(ELEMENT_LOSS);
    let dodge_loss = properties.query_property(DODGE_LOSS);
    let def_coefficient = properties.query_property(DEF_COEFFICIENT);
    let def_loss = properties.query_property(DEF_LOSS);
    let state_time = properties.query_property(STATE_TIME);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CPoisonFogPhalanx::new(
        id, master, started, lifetime, level, state_time, def_loss, def_coefficient,
        dodge_loss, element_loss, er_coefficient, weapon,
    );
    phalanx.set_center(destination.0, destination.1);
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    if let Some(Ok(id)) = game.add_poison_fog_phalanx(region_id, phalanx, destination.0, destination.1, started, runtime) {
        let _ = game.send_poison_fog_phalanx_entry(region_id, id, runtime);
    }
}
