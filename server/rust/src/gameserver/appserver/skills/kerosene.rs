//! Горючая смесь CKerosene (0xF1).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/kerosene.cpp.
//! Общий combustioncast обслуживает Begin, Check, MP, путь и End. После
//! visual1 один MasterInfo захватывается до PK и остаётся общим для всех целей.
//! Регион S фиксируется до callbacks основной цели; PK использует его карту
//! и свежие координаты захваченного U. Страна MasterInfo остаётся нулевой.
//!
//! Каждая цель проходит PK → End первого непустого ID0xF1 → destructor свежего
//! остатка → CONST/FREQUENCY/PERSIST из таблицы начала AI → новый Begin(U,S)
//! → append. Нет допуска атаки, проверки смерти соседей или UpdateProperty.
//! Основная цель обрабатывается до чтения её центра X/Y. Затем живой single
//! GetShape идёт X-снаружи/Y-внутри по включительным границам региона ±1.
//! Исключаются числовые ID U/S без учёта типа; принимаются только 400/600..603.
//! Выбор первой фигуры клетки не заменяется перебором всех или снимком области.
//!
//! Visual читает свежие U/S; отсутствие S отменяет лишь пакет выпуска, а не
//! базовый visual-tail. Around требует действительной связи U с регионом.
//! Invalid float координата сохраняет native FISTP sentinel i32::MIN.
//! Vec и state-арена заменяют native указатели, не копируя состояния. После
//! основной цели отсутствие региона безопасно прекращает только обход соседей
//! вместо native NULL-разыменования; общий caller всё равно завершает End(1).

use super::kerosenestate::{KeroseneState, begin_primary_kerosene_state};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use super::weaponattack::source_master;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, RegionShapeResolver};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;

pub(crate) const KEROSENE_SKILL_ID: u32 = 0xf1;
const PLAYER_TYPE: i32 = 400;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;

fn apply_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    region: Option<i32>, master: MasterInfo, properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) {
    if source.1.object_type == PLAYER_TYPE && target.1.object_type == PLAYER_TYPE
        && let Some(region) = region
        && let Some(user) = resolve_state_move_shape(game, source.0, source.1)
    {
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let _ = game.player_on_first_skill_at_position(
            source.1.id, target.1.id, region, source_x, source_y, runtime,
        );
    }
    if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == KEROSENE_SKILL_ID))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let hp_loss = properties.query_property(SKILL_USAGE_CONST);
    let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let keep = properties.query_property(STATE_PERSIST_TIME);
    let state = KeroseneState::new(master, keep, frequency, hp_loss);
    let _ = begin_primary_kerosene_state(
        game, target.0, target.1, Some(source), Some(target), state, None,
        &mut || runtime.now_milliseconds(),
    );
}

pub(super) fn apply_kerosene<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let region = resolve_state_move_shape(game, target.0, target.1)
        .filter(|shape| shape.shape().is_assigned_to_server_region())
        .map(|shape| shape.shape().get_region_id())
        .filter(|region| game.find_region(*region).is_some());
    apply_state(game, source, target, region, master, properties, runtime);
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let center_x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
    let center_y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
    let Some(region) = region else { return; };
    let Some((width, height)) = game.find_region(region)
        .map(|owner| (owner.base().region.width, owner.base().region.height))
    else { return; };
    let left = center_x.wrapping_sub(1).max(0);
    let top = center_y.wrapping_sub(1).max(0);
    let right = center_x.wrapping_add(1).min(width);
    let bottom = center_y.wrapping_add(1).min(height);
    for x in left..=right {
        for y in top..=bottom {
            let (area_width, area_height) = game.area_dimensions();
            let target = game.find_region(region).and_then(|owner| {
                let resolver = RegionShapeResolver { game, owner };
                owner.base().get_shape(x, y, area_width, area_height, &resolver).ok().flatten()
            }).filter(|view| view.identity.id != target.1.id && view.identity.id != source.1.id
                && matches!(view.identity.object_type, PLAYER_TYPE | 600..=603))
                .and_then(|view| resolve_state_move_shape(game, region, view.identity))
                .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
            if let Some(target) = target {
                apply_state(game, source, target, Some(region), master, properties, runtime);
            }
        }
    }
}

pub(crate) fn publish_kerosene_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CKerosene
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Kerosene || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let target = if mode == 1 {
        let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
        let Some(shape) = resolve_state_move_shape(game, region, identity) else { return; };
        Some(shape.shape())
    } else { None };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
    } else { message.add_long(source.get_direction()); }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
