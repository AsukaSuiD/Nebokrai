//! Общая подтверждённая часть фронтальных ударов мечом.
//!
//! `CJuCut`, четыре варианта `CLightningSword` и `CInverseChopped` совпадают
//! в packet layout и физико-элементно-духовной формуле. Обратный рубящий удар
//! отдельно умножает каждый компонент на накопленную энергию: оригинал усекает
//! `double` в 64-битное целое и сохраняет младшие 32 бита. Все варианты также
//! усекают результат критического множителя `float` к `i32`. Этот owner не
//! хранит execution-state и не выбирает момент списания ресурсов: различающиеся
//! lifecycle, выбор целей и ошибки остаются в конкретных skill-owner-ах.
//! Общий `End` возвращает движение, один раз выполняет унаследованный
//! `AfterUseSkill` с износом оружия и затем `CSummonSkill::End(1)`.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
pub(crate) const MONSTER_TYPE: i32 = 600;
const BATTLE_FAIRY_TYPE: i32 = 1200;

#[derive(Clone, Copy)]
pub(crate) struct FrontCellSwordDefinition {
    pub(crate) skill_id: u32,
    pub(crate) weapon_category: i32,
    pub(crate) weapon_failure_string: &'static [u8],
}

pub(crate) fn finish_front_cell_sword<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    game.damage_player_weapon(player_id, runtime);
    finish_summon_skill(game, player_id, player_ai, runtime, mark_used);
}

pub(crate) fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

pub(crate) fn weapon_is_compatible(
    game: &CGame,
    player: &CPlayer,
    definition: FrontCellSwordDefinition,
) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1)
            == definition.weapon_category
    })
}

pub(crate) fn send_failure(
    game: &CGame,
    player_id: i32,
    definition: FrontCellSwordDefinition,
    code: u8,
    mp_loss: u32,
) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, definition.weapon_failure_string),
        _ => {}
    }
}

pub(crate) fn destination(
    game: &CGame,
    region_id: i32,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(Option<ShapeIdentity>, i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((None, x, y)),
        PlayerSkillDispatch::Object { target, .. } => {
            let view = game.base_magic_target_view(region_id, target)?;
            Some((Some(target), view.tile_x, view.tile_y))
        }
        PlayerSkillDispatch::SelfTarget { .. } => {
            let face = game.find_player(player_id)?.shape().get_face_position().ok()?;
            Some((None, face.x, face.y))
        }
    }
}

pub(crate) fn send_visual(
    game: &mut CGame,
    player_id: i32,
    definition: FrontCellSwordDefinition,
    level: i32,
    dispatch: PlayerSkillDispatch,
    action: u8,
) {
    let Some((region_id, direction)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().get_direction()))
    }) else {
        return;
    };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(definition.skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(direction);
    } else {
        let (target, x, y) =
            destination(game, region_id, player_id, dispatch).unwrap_or((None, 0, 0));
        message.add_long(target.map_or(0, |identity| identity.object_type));
        message.add_long(target.map_or(0, |identity| identity.id));
        message.add_long(x);
        message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn front_shape(game: &CGame, region_id: i32, player_id: i32) -> Option<ShapeView> {
    let face = game.find_player(player_id)?.shape().get_face_position().ok()?;
    let region = game.find_region(region_id)?.base();
    let (area_width, area_height) = game.area_dimensions();
    let mut shapes = Vec::new();
    region
        .get_shapes(face.x, face.y, area_width, area_height, game, &mut shapes)
        .ok()?;
    shapes.into_iter().find(|shape| {
        matches!(
            shape.identity.object_type,
            PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE | BATTLE_FAIRY_TYPE
        )
    })
}

pub(crate) fn target_level(
    game: &CGame,
    region_id: i32,
    target: ShapeIdentity,
) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
            let monster = owner.base().find_monster_by_id(target.id)?;
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
                .map(|property| property.level as u8)
        }),
        _ => None,
    }
}

pub(crate) fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    definition: FrontCellSwordDefinition,
    target_level: u8,
    level: i32,
    hit_modifier: i32,
    target_damage_factor: u32,
) -> Option<(MasterInfo, AttackInformation)> {
    calculate_attack_with_multiplier(game, player_id, definition, target_level, level, hit_modifier, target_damage_factor, 1.0)
}

#[allow(clippy::too_many_arguments, reason = "параметры прямо соответствуют подтверждённой формуле удара")]
pub(crate) fn calculate_attack_with_multiplier(
    game: &mut CGame,
    player_id: i32,
    definition: FrontCellSwordDefinition,
    target_level: u8,
    level: i32,
    hit_modifier: i32,
    target_damage_factor: u32,
    damage_multiplier: f64,
) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let weapon_level = player.weapon_damage_level(game.goods_factory());
    let (weapon_divisor, weapon_minimum) = game.globe_setup().weapon_damage_factors();
    let level_delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let weapon_factor = if weapon_divisor == 0.0 {
        1.0
    } else {
        (level_delta as f32 / weapon_divisor)
            .min(1.0)
            .max(weapon_minimum)
    };
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let width = maximum
        .wrapping_sub(minimum)
        .wrapping_abs()
        .wrapping_add(1);
    let physical = minimum
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(combat.dexterity as i32)
        .max(0);
    let scale = |damage: i32| (f64::from(damage) * damage_multiplier) as i64 as i32;
    let mut attack = AttackInformation {
        skill_id: definition.skill_id,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor: target_damage_factor as f32 * weapon_factor * 0.01,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: scale(physical),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: scale((combat.add_element_attack as i32).max(0)),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: scale(i32::from(combat.add_soul_attack)),
                mp_damage: 0,
            },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = (power.hp_damage as f32 * critical_rate) as i32;
        }
    }
    Some((master, attack))
}
