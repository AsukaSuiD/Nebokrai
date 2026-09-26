//! Владелец одноцелевой молнии `CYunShengLightning` (`0x19E`). Источник:
//! точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) + `GameServer.pdb`
//! (RSDS match), исходный владелец `appserver/skills/yunshenglightning.cpp`.
//! Payload исполнений — `skills/execution/payload.rs`
//! (`PlayerYunShengLightningExecutionState`/`YunShengLightningProgress`);
//! общий End `0x0057B810` (ICF со SpiderWeb).
//!
//! Числовые правила, формулы и wire-кадры visual
//! `0x000BFE01` обеих ветвей (player/monster): постоянное время полёта, один
//! RNG-вызов elemental-формулы, x87-прибавка игрока с f32-константой и
//! усечением к нулю, нулевой `base_element` монстра (Windows
//! `CMonster::GetAddElementAtk` возвращает ноль даже для приручённого;
//! поэтому остаётся ровно один RNG-вызов). Нулевой `SKILL_USAGE_USER_MP_LOSE`
//! отклоняет player-cast, как исходный `CheckCastCondition`, а не превращает
//! его в бесплатный навык. Player-выпуск устанавливает prepared после
//! эффекта 1 (`0x0053B373`).
//!
//! Hub-утяжеление остаётся у делегата старого пакета и здесь не переносится:
//! зарегистрированный цикл монстра (`begin/advance/finish_base_attack_cast`,
//! `ServerRegionOwner` и его публикация), подход к дистанции и attack
//! interval (`approach_attack_range`/`schedule_attack_interval`), публикация
//! настоящего `CPlayerAI`, `finish_summon_skill` и доставка кадров
//! (`send_game_shape_around`/`send_player_shape_around`). AI монстра читает
//! reuse после очистки полёта (`CSkill::End`, `0x004D84C0`); время нанесения
//! удара не подменяет эти часы завершения.

use crate::app::game_message::CMessage;
use crate::combat::truncate_original;
use crate::regions::ShapeIdentity;

use super::PlayerSkillDispatch;

pub const YUNSHENG_LIGHTNING_SKILL_ID: u32 = 0x19e;

pub const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
pub const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
pub const SKILL_USAGE_TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;
pub const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub const SKILL_USAGE_ELEMENT_MODIFIER: u32 = 20_015;

/// Диспетчерская форма player-cast: только Point/Object этого навыка.
pub const fn is_player_yunsheng_lightning_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: YUNSHENG_LIGHTNING_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: YUNSHENG_LIGHTNING_SKILL_ID, .. })
}

/// Точка назначения player-cast: Point несёт клетку сам; Object берёт живой
/// вид цели (резолв — hub `base_magic_target_view`), иначе сохранённая точка;
/// SelfTarget не имеет назначения.
pub const fn player_destination(
    dispatch: PlayerSkillDispatch,
    object_view: Option<(i32, i32)>,
    fallback: Option<(i32, i32)>,
) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { .. } => match object_view {
            Some(view) => Some(view),
            None => fallback,
        },
        PlayerSkillDispatch::SelfTarget { .. } => None,
    }
}

/// Объектная цель player-cast (только Object несёт identity).
pub const fn player_target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        _ => None,
    }
}

/// Кадр начала `0x000BFE01`: action 1, навык, уровень, сторона, direction.
pub fn yunsheng_lightning_start_message(
    skill_level: i32,
    object_type: i32,
    object_id: i32,
    direction: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(object_type);
    message.add_long(object_id);
    message.add_long(direction);
    message
}

/// Кадр выпуска `0x000BFE01`: action 2, навык, уровень, сторона, целевая пара
/// (нули без живой цели), клетка назначения.
pub fn yunsheng_lightning_fire_message(
    skill_level: i32,
    object_type: i32,
    object_id: i32,
    target: Option<ShapeIdentity>,
    target_x: i32,
    target_y: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(object_type);
    message.add_long(object_id);
    message.add_long(target.map_or(0, |target| target.object_type));
    message.add_long(target.map_or(0, |target| target.id));
    message.add_long(target_x);
    message.add_long(target_y);
    message
}

/// Ширина elemental-диапазона `|max − min| + 1` беззнаково.
pub fn yunsheng_lightning_span(minimum: i32, maximum: i32) -> i32 {
    maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32
}

/// Стихийный урон игрока: прибавка остаётся в x87 до усечения к нулю,
/// property загружается как беззнаковый DWORD, modifier игрока — как знаковый.
pub fn yunsheng_lightning_player_element(
    add_element_attack: i32,
    minimum: i32,
    random: i32,
    element_modifier: u32,
    element_modify: f32,
) -> i32 {
    let modifier = truncate_original(
        f64::from(element_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    add_element_attack
        .wrapping_add(minimum)
        .wrapping_add(random)
        .wrapping_add(modifier)
        .max(0)
}

/// Стихийный урон монстра: `base_element` подтверждён нулём (один RNG-вызов),
/// итог не опускается ниже нуля.
pub fn yunsheng_lightning_monster_element(base_element: i32, skill_element: i32) -> i32 {
    base_element.wrapping_add(skill_element).max(0)
}
