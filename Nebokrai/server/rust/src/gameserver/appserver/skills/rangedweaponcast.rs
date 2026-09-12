//! Общие проверки и расход MP для совместимых лучных и арбалетных casts.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/meteorarrow.cpp,
//! meteorarrowmass.cpp, rainarrow.cpp, lightingarrow.cpp, lightingarrow2.cpp,
//! poisonmoth.cpp, bloodrose.cpp и explosivearrow{,2,3}.cpp.
//! Check удерживает исходного U, читает reuse и свежий путь по политике навыка.
//! Проверка самонацеливания, если она нужна, выполняется caller-ом раньше.
//! Non-player проходит без Move0; игроку нужны категория 3/4, ненулевая MP-цена
//! и неотрицательная signed DWORD-разность. MP0 — тихий отказ.
//! Первый AI использует сохранённую таблицу: MP→OnChangeStates→повторная
//! проверка оружия; поздний отказ не возвращает MP. CAN и фаза остаются
//! у игрового caller-а. ExplosiveArrow2/3 требуют лук, но сообщают GS0293
//! при отсутствии оружия и GS0286 при самонацеливании. Различия категории
//! и строк не дублируют механизм.

use super::basemagic::{SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::kernel::skill_is_restored;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RangedWeaponKind { Bow, Crossbow, ExplosiveBow }

impl RangedWeaponKind {
    const fn category(self) -> i32 {
        match self { Self::Bow | Self::ExplosiveBow => 3, Self::Crossbow => 4 }
    }
}

pub(super) fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(super) fn ranged_weapon_failure(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, mode: u32, weapon: RangedWeaponKind,
) {
    game.update_registered_skill_visual(instance, mode);
    let Some(player) = player else { return; };
    let text: &[u8] = match mode {
        4 => b"GS0300",
        10 => if weapon == RangedWeaponKind::Bow { b"GS0285" } else { b"GS0286" },
        11 => b"GS0290", 13 => b"GS0278",
        14 => if weapon == RangedWeaponKind::Crossbow { b"GS0293" } else { b"GS0297" },
        15 => b"GS0282", _ => return,
    };
    game.send_skill_system_info(player, text);
}

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player, b"GS0288", amount);
}

fn check_weapon(game: &mut CGame, instance: RegisteredSkill, player: i32, weapon: RangedWeaponKind) -> bool {
    let missing: &[u8] = if weapon == RangedWeaponKind::Bow { b"GS0297" } else { b"GS0293" };
    let wrong: &[u8] = if weapon == RangedWeaponKind::Crossbow { b"GS0293" } else { b"GS0297" };
    let failure = match game.find_player(player).and_then(|user| user.equipment().get_goods(2)) {
        None => Some(missing),
        Some(goods) => (goods.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1)
            != weapon.category()).then_some(wrong),
    };
    if let Some(text) = failure {
        game.update_registered_skill_visual(instance, 14);
        game.send_skill_system_info(player, text);
        return false;
    }
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArrowCastPathRule {
    None,
    DistanceOnly,
    DistanceAndBlocks,
}

pub(super) fn check_ranged_weapon_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity),
    path_rule: ArrowCastPathRule, weapon: RangedWeaponKind, runtime: &mut Runtime,
) -> bool {
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        ranged_weapon_failure(game, instance, player, 13, weapon);
        return false;
    }
    if path_rule != ArrowCastPathRule::None {
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
            if path.len() as u32 > maximum {
                ranged_weapon_failure(game, instance, player, 11, weapon);
                return false;
            }
        }
        if path_rule == ArrowCastPathRule::DistanceAndBlocks && path.iter().any(|cell| cell.2 == 2) {
            ranged_weapon_failure(game, instance, player, 15, weapon);
            return false;
        }
    }
    let Some(player) = player else { return true; };
    if !check_weapon(game, instance, player, weapon) { return false; }
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let Some(mana) = game.find_player(player).map(CPlayer::mana) else { return false; };
    if (mana.wrapping_sub(properties.query_property(USER_MP_LOSE)) as i32) < 0 {
        mana_failure(game, instance, player, &properties);
        return false;
    }
    let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    source.set_moveable(false);
    true
}

pub(super) fn prepare_ranged_weapon_player(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, properties: &CSkillBaseProperties,
    weapon: RangedWeaponKind,
) -> bool {
    let Some(player) = player else { return true; };
    let Some(mana) = game.find_player(player).map(CPlayer::mana) else { return false; };
    let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
    if (remaining as i32) < 0 {
        mana_failure(game, instance, player, properties);
        return false;
    }
    let Some(user) = game.find_player_mut(player) else { return false; };
    user.set_mana(remaining);
    game.publish_player_states(player);
    check_weapon(game, instance, player, weapon)
}
