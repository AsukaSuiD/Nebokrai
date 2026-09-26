//! Двухударная быстрая атака `CMonsterFastAttack` (ID `0x2d1`): константы
//! сроков, wire-кадр выпуска и машинная база исполнения. Монстр проходит
//! через hub `execute_owned_monster_base_attack` старого пакета;
//! player-dispatch, отмена и завершение используют общий двухударный owner
//! `lordfastattack` с отдельными формулой, MP и cooldown (player-ветвь fast
//! остаётся у `lordfastattack` — связка `SKILL_USAGE_FIRST_TIME`/
//! `SKILL_USAGE_SECOND_TIME` сохранена здесь для будущего lord-владельца).
//!
//! Машинные quirks: нулевая MP-цена в Check — молчаливый ret 0; удары по
//! кумулятивным срокам от начала cast (`delay+15001`, далее `+15002`);
//! мёртвая S или U==S → End(0) БЕЗ reuse; Attack без проверок U==S/500/
//! IsAttackAble и без IncreaseRp.
//!
//! Швы: состояние двух ударов (`MonsterFastAttackProgress`) живёт в
//! `skills/execution`; исполнение монстра и оркестрация hub — у владельца
//! старого пакета.
//!
//! Исходный владелец PDB: `appserver/skills/monsterfastattack.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#monsterfastattack--cmonsterfastattack-0x2d1

use crate::app::game_message::CMessage;

pub const MONSTER_FAST_ATTACK_SKILL_ID: u32 = 0x2d1;

/// Первый кумулятивный срок удара после начальной задержки (`QueryProperty`
/// 15001): `delay + first_time <= now` от начала cast.
pub const SKILL_USAGE_FIRST_TIME: u32 = 15_001;

/// Второй кумулятивный срок: `delay + first_time + second_time <= now`.
pub const SKILL_USAGE_SECOND_TIME: u32 = 15_002;

/// Кадр выпуска `0x000BFE01` монстровой быстрой атаки: action 2, навык,
/// уровень, сторона, два нулевых long и координаты цели тика (как у
/// message-owner семьи; прежний конструктор владельца).
pub fn fast_attack_fire_message(
    skill_id: u32,
    skill_level: u16,
    monster_id: i32,
    target_x: i32,
    target_y: i32,
) -> CMessage {
    const MONSTER_TYPE: i32 = 600;
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    message
}
