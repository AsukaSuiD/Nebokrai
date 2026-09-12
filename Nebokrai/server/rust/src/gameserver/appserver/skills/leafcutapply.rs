//! Наложение трёх периодических рассечений LeafCut.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/leafcut.cpp,
//! leafcut2.cpp и leafcut3.cpp.
//!
//! MasterInfo фиксируется до PK, country остаётся нулевым. PK использует
//! фактический регион S и живые координаты U. Таблица свойств удерживается
//! от начала AI, а боевые getter-ы читаются в момент создания состояния.
//! Первый вариант создаёт снимок до End старого состояния и сохраняет слот;
//! второй и третий сначала выполняют End/destructor и затем добавляют новый
//! снимок в конец. Begin, часы и DB-запись принадлежат periodicattack.
//! Отказ наложения не отменяет последующие IncreaseRp и End(1) у caller-а.
//! Nonplayer проходит Check, но native создание снимка разыменовывает результат
//! непроверенного приведения U к CPlayer. Rust не создаёт состояние на этой
//! недопустимой ветке, не подставляя фиктивный уровень оружия.

use super::leafcutstate::{LEAF_CUT_STATE_ID, LeafCutState, begin_primary_leaf_cut_state};
use super::leafcutstate2::LEAF_CUT_2_STATE_ID;
use super::leafcutstate3::LEAF_CUT_3_STATE_ID;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::AppliedState;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{end_and_destroy_state_at, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const PLAYER_TYPE: i32 = 400;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const WEAPON_DAMAGE_LEVEL_MODIFIER: u32 = 20_018;

fn master_info(game: &CGame, source: (i32, ShapeIdentity)) -> Option<MasterInfo> {
    let identity = resolve_state_move_shape(game, source.0, source.1)?.shape().identity();
    let mut master = MasterInfo {
        master_type: identity.object_type,
        master_id: identity.id,
        ..MasterInfo::default()
    };
    if identity.object_type == PLAYER_TYPE {
        let player = game.find_player(identity.id)?;
        master.master_team_id = player.team_id();
        master.master_guild_id = player.faction_id();
        master.master_union_id = player.union_id();
        let permissions = player.pk_permissions();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
    }
    Some(master)
}

fn on_first_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) {
    if source.1.object_type != PLAYER_TYPE || target.1.object_type != PLAYER_TYPE { return; }
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if !sufferer.shape().is_assigned_to_server_region() { return; }
    let target_region = sufferer.shape().get_region_id();
    if game.find_region(target_region).is_none() { return; }
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
    let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
    let _ = game.player_on_first_skill_at_position(
        source.1.id, target.1.id, target_region, source_x, source_y, runtime,
    );
}

fn new_leaf_cut_state<const ID: u32>(
    game: &CGame, source: (i32, ShapeIdentity), master: MasterInfo,
    properties: &CSkillBaseProperties,
) -> Option<LeafCutState<ID>> {
    // Четыре боевых getter-а виртуальны у CMoveShape, но последующий прямой
    // CPlayer::GetWeaponDamageLevel без проверки читает equipment через
    // результат dynamic_cast. Этот guard заменяет native NULL-разыменование,
    // а не дополнительное условие Check.
    if source.1.object_type != PLAYER_TYPE { return None; }
    resolve_state_move_shape(game, source.0, source.1)?;
    let player = game.find_player(source.1.id)?;
    let soul = player.combat_properties().add_soul_attack;
    let element = player.combat_properties().add_element_attack as u16;
    let maximum = player.combat_properties().maximum_attack as u16;
    let minimum = player.combat_properties().minimum_attack as u16;
    let weapon_modifier = properties.query_property(WEAPON_DAMAGE_LEVEL_MODIFIER) as f32;
    let weapon_level = player.weapon_damage_level(game.goods_factory()) as u32;
    // Только модификатор предварительно записан в float. Unsigned уровень
    // и factor сохраняют точность до умножения на исходную константу 0.01f.
    let weapon_factor = (
        f64::from(weapon_level) * f64::from(weapon_modifier) * f64::from(0.01_f32)
    ) as f32;
    let factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    let damage_factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
    let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let keep = properties.query_property(STATE_PERSIST_TIME);
    Some(LeafCutState::new(
        master, keep, frequency, damage_factor, weapon_factor, minimum, maximum, element, soul,
    ))
}

fn remove_previous(
    game: &mut CGame, target: (i32, ShapeIdentity), skill_id: u32, replace_in_place: bool,
) -> Option<(usize, usize)> {
    let shape = resolve_state_move_shape(game, target.0, target.1)?;
    let (position, key) = shape.find_state_position(|state| state.state_id() == skill_id)?;
    let placement = replace_in_place.then(|| shape.applied_state_replacement_location(key)).flatten();
    let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    placement
}

fn apply_leaf_cut<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) where LeafCutState<ID>: AppliedState {
    let Some(master) = master_info(game, source) else { return; };
    on_first_skill(game, source, target, runtime);
    let (state, placement) = if ID == LEAF_CUT_STATE_ID {
        let Some(state) = new_leaf_cut_state::<ID>(game, source, master, properties) else { return; };
        let placement = remove_previous(game, target, ID, true);
        (state, placement)
    } else {
        let _ = remove_previous(game, target, ID, false);
        let Some(state) = new_leaf_cut_state::<ID>(game, source, master, properties) else { return; };
        (state, None)
    };
    let _ = begin_primary_leaf_cut_state(
        game, target.0, target.1, Some(source), Some(target), state, placement,
        &mut || runtime.now_milliseconds(),
    );
}

pub(super) fn apply_leaf_cut_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), ai_properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    let Some(skill_id) = game.registered_skill(instance).map(|skill| skill.id()) else { return; };
    match skill_id {
        LEAF_CUT_STATE_ID => apply_leaf_cut::<LEAF_CUT_STATE_ID, Runtime>(game, source, target, ai_properties, runtime),
        LEAF_CUT_2_STATE_ID => apply_leaf_cut::<LEAF_CUT_2_STATE_ID, Runtime>(game, source, target, ai_properties, runtime),
        LEAF_CUT_3_STATE_ID => apply_leaf_cut::<LEAF_CUT_3_STATE_ID, Runtime>(game, source, target, ai_properties, runtime),
        _ => {}
    }
}
