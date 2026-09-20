//! Общие оружейные проверки и MP-контракт совместимых навыков.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/meteorarrow.cpp,
//! meteorarrowmass.cpp, rainarrow.cpp, lightingarrow.cpp, lightingarrow2.cpp,
//! poisonmoth.cpp, bloodrose.cpp, explosivearrow{,2,3}.cpp, scorpion.cpp,
//! boalock.cpp, strike.cpp, kerosene.cpp, ignition.cpp, heartlessarrow{,2,3}.cpp,
//! firebolt.cpp, fireball.cpp и godpunishment.cpp.
//! Проверки пути и MP доступны также
//! BoaLock/Strike без требования к оружию.
//! Check удерживает исходного U, читает reuse и свежий путь по политике навыка.
//! Проверка самонацеливания, если она нужна, выполняется caller-ом раньше.
//! Non-player проходит без Move0; игроку нужны категория 3/4, ненулевая MP-цена
//! и неотрицательная signed DWORD-разность. MP0 — тихий отказ.
//! Первый AI использует сохранённую таблицу: MP→OnChangeStates→повторная
//! проверка оружия; поздний отказ не возвращает MP. CAN и фаза остаются
//! у игрового caller-а. ExplosiveArrow2/3 требуют лук, но сообщают GS0293
//! при отсутствии оружия и GS0286 при самонацеливании. Различия категории
//! и строк не дублируют механизм.
//! Именованная ошибка препятствия читает имя захваченной caller-ом цели
//! после visual15; путь при этом может использовать уже изменённую базовую S.
//! Kerosene/Ignition читают MAX один раз и трактуют ноль как строгий предел;
//! в их Check нулевой MP допускает Move0 без чтения маны. Эти различия
//! задаются отдельно от обычного ненулевого ограничения и обязательной цены.
//! У HeartLessArrow2/3 отсутствие арбалета даёт GS0297, неверная категория —
//! GS0293. У заряжаемого HeartLessArrow поздняя проверка лука различает
//! отсутствие GS0297 и неверную категорию GS0292; первоначальная использует
//! GS0297 для обоих отказов. Варианты не меняют порядок чтений экипировки/MP.
//! FireBolt/GodPunishment используют тот же MP-допуск без финального Move0; FireBall
//! сохраняет запрет движения после достаточного положительного MP.
//! SnowStorm использует те же signed MP-проверки, но сообщает только visual7:
//! повторного запроса цены для GS0288 у него нет.
//! FireWall запрещает BLOCK1 и BLOCK2; остальные проверки препятствий
//! сохраняют исходный запрет только BLOCK2.

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
pub(super) enum RangedWeaponKind { Bow, Crossbow, ExplosiveBow, HeldBow, HeartlessCrossbow }

impl RangedWeaponKind {
    const fn category(self) -> i32 {
        match self {
            Self::Bow | Self::ExplosiveBow | Self::HeldBow => 3,
            Self::Crossbow | Self::HeartlessCrossbow => 4,
        }
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

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player: i32, properties: &CSkillBaseProperties, with_text: bool) {
    game.update_registered_skill_visual(instance, 7);
    if with_text {
        let amount = properties.query_property(USER_MP_LOSE);
        game.send_skill_system_info_with_unsigned(player, b"GS0288", amount);
    }
}

fn check_weapon(game: &mut CGame, instance: RegisteredSkill, player: i32, weapon: RangedWeaponKind) -> bool {
    let missing: &[u8] = match weapon {
        RangedWeaponKind::Bow | RangedWeaponKind::HeldBow | RangedWeaponKind::HeartlessCrossbow => b"GS0297",
        RangedWeaponKind::Crossbow | RangedWeaponKind::ExplosiveBow => b"GS0293",
    };
    let wrong: &[u8] = match weapon {
        RangedWeaponKind::Crossbow | RangedWeaponKind::HeartlessCrossbow => b"GS0293",
        RangedWeaponKind::HeldBow => b"GS0292",
        RangedWeaponKind::Bow | RangedWeaponKind::ExplosiveBow => b"GS0297",
    };
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

#[derive(Clone, Copy)]
pub(super) enum CastPathBlock {
    Ignore,
    Generic,
    GroundAndFly,
    Named { target: (i32, ShapeIdentity), message: &'static [u8] },
}

#[derive(Clone, Copy)]
pub(super) enum CastDistanceLimit { Nonzero, IncludingZero }

pub(super) fn check_skill_path(
    game: &mut CGame, instance: RegisteredSkill, properties: &CSkillBaseProperties,
    path: &[(i32, i32, u8)], player: Option<i32>, block: CastPathBlock,
) -> bool {
    check_skill_path_with_limit(game, instance, properties, path, player, CastDistanceLimit::Nonzero, block)
}

pub(super) fn check_skill_path_with_limit(
    game: &mut CGame, instance: RegisteredSkill, properties: &CSkillBaseProperties,
    path: &[(i32, i32, u8)], player: Option<i32>, limit: CastDistanceLimit, block: CastPathBlock,
) -> bool {
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let maximum = match limit {
        CastDistanceLimit::IncludingZero => Some(maximum),
        CastDistanceLimit::Nonzero if maximum != 0 => Some(properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE)),
        CastDistanceLimit::Nonzero => None,
    };
    if maximum.is_some_and(|maximum| path.len() as u32 > maximum) {
        game.update_registered_skill_visual(instance, 11);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0290"); }
        return false;
    }
    let blocked = match block {
        CastPathBlock::Ignore => false,
        CastPathBlock::GroundAndFly => path.iter().any(|cell| matches!(cell.2, 1 | 2)),
        _ => path.iter().any(|cell| cell.2 == 2),
    };
    if blocked {
        game.update_registered_skill_visual(instance, 15);
        if let Some(player) = player {
            match block {
                CastPathBlock::Generic | CastPathBlock::GroundAndFly => game.send_skill_system_info(player, b"GS0282"),
                CastPathBlock::Named { target, message } => {
                    if let Some(target) = resolve_state_move_shape(game, target.0, target.1) {
                        game.send_skill_system_info_with_text(player, message, target.shape().base_object().get_name());
                    }
                }
                CastPathBlock::Ignore => {}
            }
        }
        return false;
    }
    true
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
        let block = if path_rule == ArrowCastPathRule::DistanceAndBlocks {
            CastPathBlock::Generic
        } else { CastPathBlock::Ignore };
        if !check_skill_path(game, instance, &properties, &path, player, block) { return false; }
    }
    check_ranged_weapon_and_mana(game, instance, source, &properties, weapon, CastManaRule::RequireCost)
}

#[derive(Clone, Copy)]
pub(super) enum CastManaRule { RequireCost, AllowFree }

pub(super) fn check_ranged_weapon_and_mana(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, weapon: RangedWeaponKind, mana_rule: CastManaRule,
) -> bool {
    if source.1.object_type != PLAYER_TYPE { return true; }
    if !check_weapon(game, instance, source.1.id, weapon) { return false; }
    check_cast_mana_with_rule(game, instance, source, properties, mana_rule, true, true)
}

pub(super) fn check_cast_mana(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
) -> bool {
    check_cast_mana_with_rule(game, instance, source, properties, CastManaRule::RequireCost, true, true)
}

pub(super) fn check_cast_mana_without_movement(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
) -> bool {
    check_cast_mana_with_rule(game, instance, source, properties, CastManaRule::RequireCost, false, true)
}

pub(super) fn check_cast_mana_without_text(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
) -> bool {
    check_cast_mana_with_rule(game, instance, source, properties, CastManaRule::RequireCost, true, false)
}

fn check_cast_mana_with_rule(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, rule: CastManaRule, lock_movement: bool, with_text: bool,
) -> bool {
    if source.1.object_type != PLAYER_TYPE { return true; }
    let player = source.1.id;
    if properties.query_property(USER_MP_LOSE) == 0 {
        if matches!(rule, CastManaRule::RequireCost) { return false; }
    } else {
        let Some(mana) = game.find_player(player).map(CPlayer::mana) else { return false; };
        if (mana.wrapping_sub(properties.query_property(USER_MP_LOSE)) as i32) < 0 {
            mana_failure(game, instance, player, properties, with_text);
            return false;
        }
    }
    if lock_movement {
        let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
        source.set_moveable(false);
    }
    true
}

pub(super) fn spend_cast_mana(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, properties: &CSkillBaseProperties,
) -> bool {
    spend_cast_mana_with_text(game, instance, player, properties, true)
}

pub(super) fn spend_cast_mana_without_text(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, properties: &CSkillBaseProperties,
) -> bool {
    spend_cast_mana_with_text(game, instance, player, properties, false)
}

fn spend_cast_mana_with_text(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, properties: &CSkillBaseProperties,
    with_text: bool,
) -> bool {
    let Some(player) = player else { return true; };
    let Some(mana) = game.find_player(player).map(CPlayer::mana) else { return false; };
    let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
    if (remaining as i32) < 0 {
        mana_failure(game, instance, player, properties, with_text);
        return false;
    }
    let Some(user) = game.find_player_mut(player) else { return false; };
    user.set_mana(remaining);
    game.publish_player_states(player);
    true
}

pub(super) fn prepare_ranged_weapon_player(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, properties: &CSkillBaseProperties,
    weapon: RangedWeaponKind,
) -> bool {
    let Some(player) = player else { return true; };
    if !spend_cast_mana(game, instance, Some(player), properties) { return false; }
    check_weapon(game, instance, player, weapon)
}
